// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./InboundChannel.sol";

contract IncentivizedInboundChannel is InboundChannel {
    //TODO: Review/check below numbers and formulas
    uint256 public MAX_PAYLOAD_BYTE_SIZE = 1000;
    uint256 public MAX_PAYLOAD_GAS_COST = 500000;
    uint256 public EXTERNAL_CALL_COST = 21000;
    uint256 public POST_PROCESSING_LEFTOVER_GAS_REQUIRED = 999;
    uint256 public MAX_GAS_PER_MESSAGE =
        EXTERNAL_CALL_COST + MAX_PAYLOAD_BYTE_SIZE + MAX_PAYLOAD_GAS_COST;

    constructor() {
        nonce = 0;
    }

    // TODO: Submit should take in all inputs required for verification,
    // including eg: _parachainBlockNumber, _parachainMerkleProof, parachainHeadsMMRProof
    function submit(Message[] memory _messages, bytes32 _commitment)
        public
        override
    {
        verifyMessages(_messages, _commitment);
        processMessages(_messages);
    }

    //TODO: verifyMessages should accept all needed proofs
    function verifyMessages(Message[] memory _messages, bytes32 _commitment)
        internal
        view
        returns (bool success)
    {
        // Prove we can get the MMRLeaf that is claimed to contain our Parachain Block Header
        // BEEFYLightClient.verifyMMRLeaf(parachainHeadsMMRProof)
        // BeefyLightClient{
        //   verifyMMRLeaf(parachainHeadsMMRProof) {
        //   MMRVerification.verifyInclusionProof(latestMMRRoot, parachainHeadsMMRProof)
        // }
        //}
        //}
        // returns mmrLeaf;

        // Prove we can get the claimed parachain block header from the MMRLeaf
        // allParachainHeadsMerkleTreeRoot = mmrLeaf.parachain_heads;
        // MerkeTree.verify(allParachainHeadsMerkleTreeRoot, ourParachainMerkleProof)
        // returns parachainBlockHeader

        // Prove that the commitment is in fact in the parachain block header
        // require(parachainBlockHeader.commitment == commitment)

        // Validate that the commitment matches the commitment contents
        require(
            validateMessagesMatchCommitment(_messages, _commitment),
            "invalid commitment"
        );

        // Require all payloads are smaller than max_payload_size
        for (uint256 i = 0; i < _messages.length; i++) {
            require(
                _messages[i].payload.length <= MAX_PAYLOAD_BYTE_SIZE,
                "message payload bytesize exceeds maximum payload size"
            );
        }

        // Require there is enough gas to play all messages
        // TODO: Test/review further.
        uint256 maxMessageProcessingCost =
            _messages.length * MAX_GAS_PER_MESSAGE;
        require(
            gasleft() >=
                maxMessageProcessingCost +
                    POST_PROCESSING_LEFTOVER_GAS_REQUIRED,
            "insufficient gas for delivery of all messages"
        );

        return true;
    }

    function processMessages(Message[] memory _messages) internal {
        for (uint256 i = 0; i < _messages.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            Message memory message = _messages[i];
            require(message.nonce == nonce, "invalid nonce");

            nonce = nonce + 1;

            // Deliver the message to the target
            // Delivery will have fixed maximum gas allowed for the target app
            (bool success, ) =
                message.target.call{value: 0, gas: MAX_GAS_PER_MESSAGE}(
                    message.payload
                );

            emit MessageDelivered(message.nonce, success);
        }
    }

    function validateMessagesMatchCommitment(
        Message[] memory _messages,
        bytes32 _commitment
    ) internal pure returns (bool) {
        bytes memory messagesBytes;
        for (uint256 i = 0; i < _messages.length; i++) {
            bytes memory messageBytes =
                abi.encodePacked(
                    _messages[i].target,
                    _messages[i].nonce,
                    _messages[i].payload
                );
            messagesBytes = abi.encodePacked(messagesBytes, messageBytes);
        }
        return keccak256(messagesBytes) == _commitment;
    }
}
