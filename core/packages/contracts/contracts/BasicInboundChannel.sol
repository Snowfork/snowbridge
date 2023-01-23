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
        bytes calldata parachainHeaderProof
    ) external {
        bytes32 commitment = MerkleProof.processProof(message, leafProof, hashSides);
        require(
            parachainClient.verifyCommitment(commitment, parachainHeaderProof),
            "Invalid proof"
        );
        require(message.nonce == nonce[message.sourceId] + 1, "Invalid nonce");
        // TODO: should we check the remaining gas?
        nonce[message.sourceId]++;
        dispatch(message);
        // TODO: should we emit a MessageDispatched event?
    }

    function submitBatch(
        Message[] calldata messages,
        bytes32[][] calldata leafProofs,
        bool[][] calldata hashSides,
        bytes calldata parachainHeaderProof
    ) external {
        bytes32 commitment = MerkleProof.processMultiProof(messages, leafProofs, hashSides);
        require(
            parachainClient.verifyCommitment(commitment, parachainHeaderProof),
            "Invalid proof"
        );
        for (uint256 i = 0; i < messages.length; i++) {
            require(messages[i].nonce == nonce[messages[i].sourceId] + 1, "Invalid nonce");
            nonce[messages[i].sourceId]++;
        }
        dispatchBatch(messages);
    }

    // solhint-disable no-empty-blocks
    function dispatch(Message calldata messages) internal {
        // TODO: dispatch to XCM interpreter
    }

    // solhint-disable no-empty-blocks
    function dispatchBatch(Message[] calldata messages) internal {
        // TODO: dispatch to XCM interpreter
    }
}
