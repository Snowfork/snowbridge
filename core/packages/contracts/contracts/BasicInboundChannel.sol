// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";
import "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

contract BasicInboundChannel {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    mapping(bytes32 => uint64) public nonce;

    ParachainClient public parachainClient;

    struct Message {
        bytes32 sourceID;
        uint64 nonce;
        bytes payload;
    }

    event MessageDispatched(bytes32 sourceID, uint64 nonce);

    constructor(ParachainClient _parachainClient) {
        parachainClient = _parachainClient;
    }

    function submit(
        Message calldata message,
        bytes32[] calldata leafProof,
        bytes calldata parachainHeaderProof
    ) external {
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);
        require(
            parachainClient.verifyCommitment(commitment, parachainHeaderProof),
            "Invalid proof"
        );
        require(message.nonce == nonce[message.sourceID] + 1, "Invalid nonce");
        require(gasleft() >= MAX_GAS_PER_MESSAGE + GAS_BUFFER, "insufficient gas");
        nonce[message.sourceID]++;
        dispatch(message);
    }

    function dispatch(Message calldata message) internal {
        emit MessageDispatched(message.sourceID, message.nonce);
    }
}
