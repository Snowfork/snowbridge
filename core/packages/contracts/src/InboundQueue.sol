// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IParachainClient, ParachainClient} from "./ParachainClient.sol";
import {Registry} from "./Registry.sol";
import {RegistryLookup} from "./RegistryLookup.sol";
import {Auth} from "./Auth.sol";
import {Vault} from "./Vault.sol";

import {IRecipient} from "./IRecipient.sol";
import {ParaID} from "./Types.sol";

contract InboundQueue is Auth, RegistryLookup {
    // Nonce for each origin
    mapping(ParaID origin => uint64) public nonce;

    // Light client message verifier
    IParachainClient public parachainClient;

    // Relayers are rewarded from this vault
    Vault public immutable vault;

    // The relayer reward for submitting a message
    uint256 public reward;

    // Relayers must provide enough gas to cover message dispatch plus a buffer
    uint256 public gasToForward = 500000;
    uint256 public constant GAS_BUFFER = 24000;

    // Inbound message from BridgeHub parachain
    struct Message {
        ParaID origin;
        uint64 nonce;
        bytes32 recipient;
        bytes payload;
    }

    enum DispatchResult {
        Success,
        Failure
    }

    event MessageDispatched(ParaID origin, uint64 nonce, DispatchResult result);
    event HandlerUpdated(uint16 id, IRecipient handler);
    event ParachainClientUpdated(address parachainClient);
    event VaultUpdated(address vault);
    event RewardUpdated(uint256 reward);
    event GasToForwardUpdated(uint256 gasToForward);
    event InvalidRecipient(bytes32 recipient);

    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();

    constructor(Registry registry, IParachainClient _parachainClient, Vault _vault, uint256 _reward)
        RegistryLookup(registry)
    {
        parachainClient = _parachainClient;
        vault = _vault;
        reward = _reward;
    }

    function submit(Message calldata message, bytes32[] calldata leafProof, bytes calldata headerProof) external {
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);
        if (!parachainClient.verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }
        if (message.nonce != nonce[message.origin] + 1) {
            revert InvalidNonce();
        }

        // Increment nonce for origin.
        // This ensures messages are not replayed. It also ensures re-entrancy protection,
        // as a re-entrant call will be detected by the nonce check above.
        //
        // Sources of re-entrancy:
        // * The relayer which gets forwarded ETH as a reward for submission
        // * XCM::Transact calls to arbitrary untrusted contracts (not in scope for initial release)
        nonce[message.origin]++;

        // reward the relayer
        // Should revert if there are not enough funds. In which case, the origin
        // should top up the funds and have a relayer resend the message.
        vault.withdraw(message.origin, payable(msg.sender), reward);

        // Ensure relayers pass enough gas for message to execute.
        // Otherwise malicious relayers can break the bridge by allowing handlers to run out gas.
        // Resubmission of the message by honest relayers will fail as the tracked nonce
        // has already been updated.
        if (gasleft() < gasToForward + GAS_BUFFER) {
            revert NotEnoughGas();
        }

        address recipient = resolve(message.recipient);
        DispatchResult result = DispatchResult.Success;
        try IRecipient(recipient).handle{gas: gasToForward}(message.origin, message.payload) {}
        catch {
            result = DispatchResult.Failure;
        }

        emit MessageDispatched(message.origin, message.nonce, result);
    }

    function updateReward(uint256 _reward) external onlyRole(ADMIN_ROLE) {
        reward = _reward;
        emit RewardUpdated(_reward);
    }

    function updateGasToForward(uint256 _gasToForward) external onlyRole(ADMIN_ROLE) {
        gasToForward = _gasToForward;
        emit GasToForwardUpdated(_gasToForward);
    }
}
