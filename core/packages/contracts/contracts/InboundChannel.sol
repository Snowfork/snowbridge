// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";
import "./utils/MerkleProof.sol";
import "./IController.sol";

contract BasicInboundChannel {
    mapping(bytes => uint64) public nonce;

    ParachainClient public parachainClient;

    struct Message {
        bytes origin;
        uint64 nonce;
        address dest;
        bytes payload;
    }

    event MessageDispatched(bytes origin, uint64 nonce);

    constructor(ParachainClient _parachainClient) {
        parachainClient = _parachainClient;
    }

    function submit(
        Message calldata message,
        bytes32[] calldata leafProof,
        bool[] calldata hashSides,
        bytes calldata parachainHeaderProof
    ) external {
        bytes32 commitment = MerkleProof.processProof(message, leafProof, hashSides);
        require(
            parachainClient.verifyCommitment(commitment, parachainHeaderProof),
            "Invalid proof"
        );
        require(message.nonce == nonce[message.origin] + 1, "Invalid nonce");
        nonce[message.origin]++;

        IController(message.dest).handle(origin, message.payload);

        emit MessageDispatched(message.origin, message.nonce);
    }

}
