// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";

contract BasicInboundChannel {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    uint64 public nonce;

    ParachainClient public parachainClient;

    struct MessageBundle {
        uint64 nonce;
        Message[] messages;
    }

    struct Message {
        uint64 id;
        address target;
        bytes payload;
    }

    event MessageDispatched(uint64 id, bool result);

    constructor(ParachainClient client) {
        nonce = 0;
        parachainClient = client;
    }

    function submit(MessageBundle calldata bundle, ParachainClient.Proof calldata proof) external {
        bytes32 commitment = keccak256(abi.encode(bundle));

        require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");

        // Require there is enough gas to play all messages
        require(
            gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );

        processMessages(bundle);
    }

    function processMessages(MessageBundle calldata bundle) internal {
        require(bundle.nonce == nonce + 1, "invalid nonce");

        for (uint256 i = 0; i < bundle.messages.length; i++) {
            Message calldata message = bundle.messages[i];

            // Deliver the message to the target
            (bool success, ) = message.target.call{ value: 0, gas: MAX_GAS_PER_MESSAGE }(
                message.payload
            );

            emit MessageDispatched(message.id, success);
        }

        nonce++;
    }
}
