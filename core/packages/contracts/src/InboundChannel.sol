// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "openzeppelin/utils/cryptography/MerkleProof.sol";
import "openzeppelin/access/AccessControl.sol";
import "./ParachainClient.sol";
import "./IRecipient.sol";
import "./ISovereignTreasury.sol";

contract InboundChannel is AccessControl {
    mapping(bytes => uint64) public nonce;
    mapping(uint16 => IRecipient) handlers;
    IParachainClient public parachainClient;
    ISovereignTreasury public sovereignTreasury;
    uint256 public reward;

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    struct Message {
        bytes origin;
        uint64 nonce;
        uint16 handler;
        bytes payload;
    }

    event MessageDispatched(bytes origin, uint64 nonce);
    event HandlerUpdated(uint16 id, address handler);
    event ParachainClientUpdated(address parachainClient);
    event SovereignTreasuryUpdated(address sovereignTreasury);
    event RewardUpdated(uint256 reward);

    error InvalidProof();
    error InvalidNonce();

    constructor(
        IParachainClient _parachainClient,
        ISovereignTreasury _sovereignTreasury,
        uint256 _reward
    ) {
        _grantRole(ADMIN_ROLE, msg.sender);
        parachainClient = _parachainClient;
        sovereignTreasury = _sovereignTreasury;
        reward = _reward;
    }

    function submit(
        Message calldata message,
        bytes32[] calldata leafProof,
        bytes calldata parachainHeaderProof
    ) external {
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(message, leafProof);
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
        sovereignTreasury.withdraw(message.origin, payable(msg.sender), reward);

        // Check if there is known handler, otherwise fail silently.
        IRecipient handler = handlers[message.handler];
        if (address(handler) == address(0)) {
            return;
        } else {
            // dispatch message
            // TODO: Should revert on out-of-gas errors. Need to verify.
            try handler.handle(message.origin, message.payload) {} catch {}
            emit MessageDispatched(message.origin, message.nonce);
        }
    }

    function updateHandler(uint16 id, IRecipient handler) external onlyRole(ADMIN_ROLE) {
        handlers[id] = handler;
        emit HandlerUpdated(id, address(handler));
    }

    function updateParachainClient(
        IParachainClient _parachainClient
    ) external onlyRole(ADMIN_ROLE) {
        parachainClient = _parachainClient;
        emit ParachainClientUpdated(address(_parachainClient));
    }

    function updateSovereignTreasury(
        ISovereignTreasury _sovereignTreasury
    ) external onlyRole(ADMIN_ROLE) {
        sovereignTreasury = _sovereignTreasury;
        emit SovereignTreasuryUpdated(address(_sovereignTreasury));
    }

    function updateReward(uint256 _reward) external onlyRole(ADMIN_ROLE) {
        reward = _reward;
        emit RewardUpdated(_reward);
    }
}
