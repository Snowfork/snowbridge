// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "./ParachainLightClient.sol";
import "./BeefyLightClient.sol";
import "./SimplifiedMMRVerification.sol";

contract BasicInboundChannel {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    uint64 public nonce;

    BeefyLightClient public beefyLightClient;

    struct MessageBundle {
        uint64 nonce;
        Message[] messages;
    }

    struct Message {
        uint64 id;
        address target;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    constructor(BeefyLightClient _beefyLightClient) {
        nonce = 0;
        beefyLightClient = _beefyLightClient;
    }

    function submit(
        MessageBundle calldata bundle,
        ParachainLightClient.ParachainVerifyInput
            calldata _parachainVerifyInput,
        ParachainLightClient.BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        SimplifiedMMRProof calldata proof
    ) public {
        // Proof
        // 1. Compute our parachain's message `commitment` by ABI encoding and hashing the `_messages`
        bytes32 commitment = keccak256(abi.encode(bundle));

        ParachainLightClient.verifyCommitmentInParachain(
            commitment,
            _parachainVerifyInput,
            _beefyMMRLeafPartial,
            proof,
            beefyLightClient
        );

        // Require there is enough gas to play all messages
        require(
            gasleft() >= (_messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );

        processMessages(_messages);
    }

    function processMessages(MessageBundle calldata bundle) internal {
        require(bundle.nonce == nonce + 1, "invalid nonce");

        for (uint256 i = 0; i < bundle.length; i++) {
            Message memory message = bundle.messages[i];

            // Deliver the message to the target
            (bool success, ) = message.target.call{
                value: 0,
                gas: MAX_GAS_PER_MESSAGE
            }(message.payload);

            emit MessageDispatched(_messages[i].id, success);
        }

        nonce++;
    }
}
