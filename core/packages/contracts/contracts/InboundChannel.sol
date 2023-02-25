// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";
import "./utils/MerkleProof.sol";
import "./IRecipient.sol";
import "./ISovereignTreasury.sol";

contract InboundChannel {
    mapping(bytes => uint64) public nonce;

    ParachainClient public parachainClient;

    ISovereignTreasury public sovereignTreasury;

    uint256 public reward;

    struct Message {
        bytes origin;
        uint64 nonce;
        address dest;
        bytes payload;
    }

    event MessageDispatched(bytes origin, uint64 nonce);

    error InvalidProof();
    error InvalidNonce();

    constructor(ParachainClient _parachainClient, ISovereignTreasury _sovereignTreasury, uint256 _reward) {
        parachainClient = _parachainClient;
        sovereignTreasury = _sovereignTreasury;
        reward = _reward;
    }

    function submit(
        Message calldata message,
        bytes32[] calldata leafProof,
        bool[] calldata hashSides,
        bytes calldata parachainHeaderProof
    ) external {
        bytes32 commitment = MerkleProof.processProof(message, leafProof, hashSides);
        if (!parachainClient.verifyCommitment(commitment, parachainHeaderProof)) {
            revert InvalidProof();
        }
        if (message.nonce != nonce[message.origin] + 1) {
            revert InvalidNonce();
        }

        nonce[message.origin]++;

        // dispatch message
        try IRecipient(message.dest).handle(message.origin, message.payload) {} catch {}

        // reward the relayer
        sovereignTreasury.withdraw(message.origin, msg.sender, reward);

        emit MessageDispatched(message.origin, message.nonce);
    }

}
