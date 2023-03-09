// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "openzeppelin/utils/cryptography/MerkleProof.sol";
import "openzeppelin/access/AccessControl.sol";
import "./ParachainClient.sol";
import "./IRecipient.sol";
import "./IVault.sol";

contract InboundChannel is AccessControl {
    mapping(bytes => uint64) public nonce;
    mapping(uint16 => IRecipient) handlers;
    IParachainClient public parachainClient;
    IVault public vault;
    uint256 public reward;

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    uint256 public constant GAS_BUFFER = 100000;

    struct Message {
        bytes origin;
        uint64 nonce;
        uint16 handler;
        uint256 gas;
        bytes payload;
    }

    event MessageDispatched(bytes origin, uint64 nonce);
    event HandlerUpdated(uint16 id, address handler);
    event ParachainClientUpdated(address parachainClient);
    event VaultUpdated(address vault);
    event RewardUpdated(uint256 reward);

    error InvalidProof();
    error InvalidNonce();
    error InvalidHandler();
    error NotEnoughGas();

    constructor(IParachainClient _parachainClient, IVault _vault, uint256 _reward) {
        _grantRole(ADMIN_ROLE, msg.sender);
        parachainClient = _parachainClient;
        vault = _vault;
        reward = _reward;
    }

    function submit(Message calldata message, bytes32[] calldata leafProof, bytes calldata parachainHeaderProof)
        external
    {
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);
        if (!parachainClient.verifyCommitment(commitment, parachainHeaderProof)) {
            revert InvalidProof();
        }
        if (message.nonce != nonce[message.origin] + 1) {
            revert InvalidNonce();
        }

        nonce[message.origin]++;

        // reward the relayer
        // Should revert if there are not enough funds. In which case, the origin
        // should top up the funds and have a relayer resend the message.
        vault.withdraw(message.origin, payable(msg.sender), reward);

        IRecipient handler = handlers[message.handler];
        if (address(handler) == address(0)) {
            revert InvalidHandler();
        }

        // Ensure relayers pass enough gas for message to execute
        if (gasleft() < message.gas + GAS_BUFFER) {
            revert NotEnoughGas();
        }

        try handler.handle{gas: message.gas}(message.origin, message.payload) {} catch {}
        emit MessageDispatched(message.origin, message.nonce);
    }

    function updateHandler(uint16 id, IRecipient handler) external onlyRole(ADMIN_ROLE) {
        handlers[id] = handler;
        emit HandlerUpdated(id, address(handler));
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
}
