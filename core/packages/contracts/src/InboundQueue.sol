// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {MerkleProofLib} from "solmate/utils/MerkleProofLib.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IParachainClient} from "./ParachainClient.sol";
import {IRecipient} from "./IRecipient.sol";
import {IVault} from "./IVault.sol";
import {ParaID} from "./Types.sol";

contract InboundQueue is AccessControl {
    // Nonce for each origin
    mapping(ParaID origin => uint64) public nonce;

    // Registered message handlers
    mapping(uint16 handlerID => IRecipient) public handlers;

    // Light client message verifier
    IParachainClient public parachainClient;

    // Relayers are rewarded from this vault
    IVault public vault;

    // The relayer reward for submitting a message
    uint256 public reward;

    // The governance contract which is a proxy for Polkadot governance, administers via this role
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    // Relayers must provide enough gas to cover message dispatch plus a buffer
    uint256 public gasToForward = 500000;
    uint256 public constant GAS_BUFFER = 24000;

    // Inbound message from BridgeHub parachain
    struct Message {
        ParaID origin;
        uint64 nonce;
        uint16 handler;
        bytes payload;
    }

    enum DispatchResult {
        Success,
        Failure
    }

    event MessageDispatched(ParaID indexed origin, uint64 indexed nonce, DispatchResult result);
    event HandlerUpdated(uint16 id, IRecipient handler);
    event ParachainClientUpdated(address parachainClient);
    event VaultUpdated(address vault);
    event RewardUpdated(uint256 reward);
    event GasToForwardUpdated(uint256 gasToForward);


    error InvalidProof();
    error InvalidNonce();
    error InvalidHandler();
    error NotEnoughGas();

    constructor(IParachainClient _parachainClient, IVault _vault, uint256 _reward) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(ADMIN_ROLE, ADMIN_ROLE);
        parachainClient = _parachainClient;
        vault = _vault;
        reward = _reward;
    }

    function submit(Message calldata message, bytes32[] calldata leafProof, bytes calldata headerProof) external {
        // Hash the leaf so that we can combine it with the proof hashes to find the Merkle root.
        bytes32 leafHash = keccak256(abi.encode(message));
        // Get the root hash that identifies the list of messages that the caller claims the parachain has committed.
        bytes32 commitment = MerkleProofLib.calculateRoot(leafProof, leafHash);
        // Verify that the parachain has committed the list of messages.
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

        IRecipient handler = handlers[message.handler];
        if (address(handler) == address(0)) {
            revert InvalidHandler();
        }

        // Ensure relayers pass enough gas for message to execute.
        // Otherwise malicious relayers can break the bridge by allowing handlers to run out gas.
        // Resubmission of the message by honest relayers will fail as the tracked nonce
        // has already been updated.
        if (gasleft() < gasToForward + GAS_BUFFER) {
            revert NotEnoughGas();
        }

        DispatchResult result = DispatchResult.Success;
        try handler.handle{gas: gasToForward}(message.origin, message.payload) {}
        catch {
            result = DispatchResult.Failure;
        }

        emit MessageDispatched(message.origin, message.nonce, result);
    }

    function updateHandler(uint16 id, IRecipient handler) external onlyRole(ADMIN_ROLE) {
        handlers[id] = handler;
        emit HandlerUpdated(id, handler);
    }

    function updateParachainClient(IParachainClient _parachainClient) external onlyRole(ADMIN_ROLE) {
        parachainClient = _parachainClient;
        emit ParachainClientUpdated(address(_parachainClient));
    }

    function updateVault(IVault _vault) external onlyRole(ADMIN_ROLE) {
        vault = _vault;
        emit VaultUpdated(address(_vault));
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
