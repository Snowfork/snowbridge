// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IInboundQueueDelegate} from "./IInboundQueueDelegate.sol";
import {IParachainClient} from "./ParachainClient.sol";
import {IRecipient} from "./IRecipient.sol";
import {IVault} from "./IVault.sol";
import {ParaID} from "./Types.sol";

contract InboundQueueDelegate is IInboundQueueDelegate, AccessControl {
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
    bytes32 public constant PROXT_ROLE = keccak256("PROXY_ROLE");

    // Relayers must provide enough gas to cover message dispatch plus a buffer
    uint256 public gasToForward = 500000;
    uint256 public constant GAS_BUFFER = 24000;

    address immutable facade;

    // Inbound message from BridgeHub parachain
    struct Message {
        ParaID origin;
        uint64 nonce;
        uint16 handler;
        bytes payload;
    }

    event MessageReceived(ParaID indexed origin, uint64 indexed nonce, bool success);
    event HandlerUpdated(uint16 id, IRecipient handler);
    event ParachainClientUpdated(address parachainClient);
    event VaultUpdated(address vault);
    event RewardUpdated(uint256 reward);
    event GasToForwardUpdated(uint256 gasToForward);

    error InvalidSender();
    error InvalidProof();
    error InvalidNonce();
    error InvalidHandler();
    error NotEnoughGas();

    constructor(address _facade, IParachainClient _parachainClient, IVault _vault, uint256 _reward) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(ADMIN_ROLE, ADMIN_ROLE);
        facade = _facade;
        parachainClient = _parachainClient;
        vault = _vault;
        reward = _reward;
    }

    function submit(address payable relayer, bytes calldata opaqueMessage) external {
        // Check that the sender is the facade (proxy)
        if (msg.sender != facade) {
            revert InvalidSender();
        }

        // Decode opaque message
        (
            Message memory message,
            bytes32[] memory leafProof,
            bytes memory headerProof
        ) = abi.decode(opaqueMessage, (Message, bytes32[], bytes));

        // Generate a merkle root (known as the commitment) from the leaf and leaf proof
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);

        // Verify that the commitment is included in a parachain header within the current MMR state.
        if (!parachainClient.verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }

        // Ensure the verified message is not being replayed
        if (message.nonce != nonce[message.origin] + 1) {
            revert InvalidNonce();
        }

        // Increment nonce for origin.
        nonce[message.origin]++;

        // Reward the relayer
        // Will revert if there are not enough funds. In which case, the origin should
        // top up the funds and have a relayer resend the message.
        vault.withdraw(message.origin, payable(relayer), reward);

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

        bool success = true;
        try handler.handle{gas: gasToForward}(message.origin, message.payload) {}
        catch {
            success = false;
        }

        emit MessageReceived(message.origin, message.nonce, success);
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
