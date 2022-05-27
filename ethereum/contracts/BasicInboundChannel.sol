// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./ParachainClient.sol";

contract BasicInboundChannel {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    uint8 public immutable sourceChannelID;

    uint64 public nonce;

    ParachainClient public parachainClient;

    struct MessageBundle {
        uint8 sourceChannelID;
        uint64 nonce;
        Message[] messages;
    }

    struct Message {
        uint64 id;
        address target;
        bytes payload;
    }

    event MessageDispatched(uint64 id, bool result);

    constructor(uint8 _sourceChannelID, ParachainClient _parachainClient) {
        nonce = 0;
        sourceChannelID = _sourceChannelID;
        parachainClient = _parachainClient;
    }

    function submit(MessageBundle calldata bundle,  bytes calldata proof) external {
        bytes32 commitment = keccak256(abi.encode(bundle));

        require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");
        require(bundle.sourceChannelID == sourceChannelID, "Invalid source channel");
        require(bundle.nonce == nonce + 1, "Invalid nonce");
        require(
            gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );
        nonce++;
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
