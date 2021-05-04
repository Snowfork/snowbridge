// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";

contract BasicInboundChannel {
    uint256 constant public MAX_GAS_PER_MESSAGE = 100000;

    struct Message {
        address target;
        uint64 nonce;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    // TODO: Submit should take in all inputs required for verification,
    // including eg: _parachainBlockNumber, _parachainMerkleProof, parachainHeadsMMRProof
    function submit(Message[] calldata _messages, bytes32 _commitment)
        public
        override
    {
        verifyMessages(_messages, _commitment);
        processMessages(_messages);
    }

    //TODO: verifyMessages should accept all needed proofs
    function verifyMessages(Message[] calldata _messages, bytes32 _commitment)
        internal
        view
        returns (bool success)
    {
        require(
            validateMessagesMatchCommitment(_messages, _commitment),
            "invalid commitment"
        );

        // Require there is enough gas to play all messages
        require(
            gasleft() >= _messages.length * MAX_GAS_PER_MESSAGE,
            "insufficient gas for delivery of all messages"
        );

        return true;
    }

    function processMessages(Message[] calldata _messages) internal {
        for (uint256 i = 0; i < _messages.length; i++) {
            // Check message nonce is correct and increment nonce for replay protection
            require(_messages[i].nonce == nonce + 1, "invalid nonce");

            nonce = nonce + 1;

            // Deliver the message to the target
            (bool success, ) =
                _messages[i].target.call{value: 0, gas: MAX_GAS_PER_MESSAGE}(_messages[i].payload);

            emit MessageDispatched(_messages[i].nonce, success);
        }
    }

    function validateMessagesMatchCommitment(
        Message[] calldata _messages,
        bytes32 _commitment
    ) internal pure returns (bool) {
        return keccak256(abi.encode(_messages)) == _commitment;
    }
}
