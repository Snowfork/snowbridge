// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";
import "./utils/MerkleProof.sol";

contract BasicInboundChannelV2 {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    uint8 public immutable sourceChannelID;

    mapping(bytes32 => uint64) public nonces;

    ParachainClient public parachainClient;

    struct MessageBundle {
        uint8 sourceChannelID;
        bytes32 account;
        uint64 nonce;
        Message[] messages;
    }

    struct Message {
        uint64 id;
        address target;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    constructor(uint8 _sourceChannelID, ParachainClient _parachainClient) {
        sourceChannelID = _sourceChannelID;
        parachainClient = _parachainClient;
    }

    function generateCommitmentHash(
        MessageBundle calldata leaf,
        bytes32[] calldata leafProof,
        bool[] calldata nodeSide
    ) internal pure returns (bytes32) {
        bytes32 leafHash = keccak256(abi.encode(leaf));
        return
            MerkleProof.computeRootFromProofAndSide(
                leafHash,
                leafProof,
                nodeSide
            );
    }

    function submit(
        MessageBundle calldata leaf,
        bytes32[] calldata leafProof,
        bool[] calldata nodeSide,
        bytes calldata proof
    ) public {
        bytes32 commitment = generateCommitmentHash(leaf, leafProof, nodeSide);

        require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");
        require(leaf.sourceChannelID == sourceChannelID, "Invalid source channel");
        require(leaf.nonce == nonces[leaf.account] + 1, "Invalid nonce");
        // Require there is enough gas to play all messages
        require(
            gasleft() >= (leaf.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );

        nonces[leaf.account]++;
        dispatch(leaf);
    }

    function dispatch(MessageBundle calldata bundle) internal {
        for (uint256 i = 0; i < bundle.messages.length; i++) {
            Message calldata message = bundle.messages[i];
            // Deliver the message to the target
            (bool success, ) = message.target.call{
                value: 0,
                gas: MAX_GAS_PER_MESSAGE
            }(message.payload);
            emit MessageDispatched(message.id, success);
        }
    }
}
