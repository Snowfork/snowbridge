// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";
import "./utils/MerkleProof.sol";

contract BasicInboundChannel {
    mapping(bytes32 => uint64) public nonce;

    ParachainClient public parachainClient;

    struct Message {
        bytes32 sourceId;
        uint64 nonce;
        bytes payload;
    }

    constructor(ParachainClient _parachainClient) {
        parachainClient = _parachainClient;
    }

    function submit(
        Message calldata message,
        bytes32[] calldata leafProof,
        bool[] calldata hashSides,
        bytes calldata proof
    ) external {
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.computeRootFromProofAndSide(
            leafHash,
            leafProof,
            hashSides
        );

        require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");
        require(message.nonce == nonce[message.sourceId] + 1, "Invalid nonce");
        // TODO: check remaining gas?
        nonce[message.sourceId]++;
        // TODO: dispatch to XCM interpreter
        // dispatch(message);
        // TODO: emit MessageDispatched event?
    }
}
