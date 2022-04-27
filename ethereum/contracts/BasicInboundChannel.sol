// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "./ParachainClient.sol";
import "./BeefyLightClient.sol";
import "./utils/MMRProofVerification.sol";

contract BasicInboundChannel {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    uint64 public nonce;

    ParachainClient public client;

    struct Message {
        address target;
        uint64 nonce;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    constructor(ParachainClient _client) {
        nonce = 0;
        client = _client;
    }

    function submit(
        Message[] calldata _messages,
        ParachainClient.Proof calldata proof
    ) external {
        bytes32 commitment = keccak256(abi.encode(_messages));

        client.verifyCommitment(
            commitment,
            proof
        );

        // Require there is enough gas to play all messages
        require(
            gasleft() >= (_messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );

        processMessages(_messages);
    }

    function processMessages(Message[] calldata _messages) internal {
        // Caching nonce for gas optimization
        uint64 cachedNonce = nonce;

        for (uint256 i = 0; i < _messages.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            require(_messages[i].nonce ==  cachedNonce + 1, "invalid nonce");

            cachedNonce = cachedNonce + 1;

            // Deliver the message to the target
            (bool success, ) = _messages[i].target.call{
                value: 0,
                gas: MAX_GAS_PER_MESSAGE
            }(_messages[i].payload);

            emit MessageDispatched(_messages[i].nonce, success);
        }
        nonce = cachedNonce;
    }
}
