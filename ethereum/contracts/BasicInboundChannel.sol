// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./InboundChannel.sol";

contract BasicInboundChannel is InboundChannel {
    uint256 public MAX_PAYLOAD_BYTE_SIZE = 1000;
    uint256 public MAX_PAYLOAD_GAS_COST = 500000;
    uint256 public EXTERNAL_CALL_COST = 21000;
    uint256 public MAX_GAS_PER_MESSAGE =
        EXTERNAL_CALL_COST + MAX_PAYLOAD_BYTE_SIZE + EXTERNAL_CALL_COST;

    constructor() {
        nonce = 0;
    }

    function submit(Message[] memory commitment) public override {
        //verifyCommitment(commitment);
        processCommitment(commitment);
    }

    function verifyCommitment(Message[] memory commitment) internal view returns (bool success) {
        // Require there is enough gas to play all messages
        require(
            gasleft() >=
                commitment.length * MAX_GAS_PER_MESSAGE,
            "insufficient gas for delivery of all messages"
        );

        // Require all payloads are smaller than max_payload_size
        for (uint256 i = 0; i < commitment.length; i++) {
            require(
                commitment[i].payload.length <=
                    MAX_PAYLOAD_BYTE_SIZE,
                "message payload bytesize exceeds maximum payload size"
            );
        }
        return true;
    }

    function processCommitment(
        Message[] memory commitment
    ) internal {
        for (uint256 i = 0; i < commitment.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            Message memory message = commitment[i];
            require(message.nonce == nonce, "invalid nonce");

            nonce = nonce + 1;

            // Delivery will have fixed maximum gas allowed for the destination app.
            // uint256 allowedGas = MAX_GAS_PER_MESSAGE;

            bool success;
            bytes memory result;
            (success, result) = message.target.call{value: 0}(message.payload);

            emit MessageDelivered(message.nonce, success);
        }
    }

}
