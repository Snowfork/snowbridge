// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";
import "./utils/MerkleProof.sol";

contract BasicInboundChannel {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    mapping(bytes32 => uint64) public nonce;

    ParachainClient public parachainClient;

    struct MessageBundle {
        bytes32 account;
        uint64 nonce;
        Message[] messages;
    }

    struct Message {
        uint64 id;
        address target;
        bytes payload;
    }

    event MessageDispatched(uint64 id, bool result);

    constructor(ParachainClient _parachainClient) {
        parachainClient = _parachainClient;
    }

    function submit(
        MessageBundle calldata bundle,
        bytes32[] calldata leafProof,
        bool[] calldata hashSides,
        bytes calldata proof
    ) external {
        bytes32 leafHash = keccak256(abi.encode(bundle));
        bytes32 commitment = MerkleProof.computeRootFromProofAndSide(
            leafHash,
            leafProof,
            hashSides
        );

        require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");
        require(bundle.nonce == nonce[bundle.account] + 1, "Invalid nonce");
        require(
            gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas"
        );
        nonce[bundle.account]++;
        dispatch(bundle);
    }

    function dispatch(MessageBundle calldata bundle) internal {
        for (uint256 i = 0; i < bundle.messages.length; i++) {
            Message calldata message = bundle.messages[i];
            (bool success, ) = message.target.call{ value: 0, gas: MAX_GAS_PER_MESSAGE }(
                message.payload
            );
            emit MessageDispatched(message.id, success);
        }
    }
}
