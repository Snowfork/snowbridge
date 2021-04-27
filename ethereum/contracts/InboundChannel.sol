// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

abstract contract InboundChannel {
    uint64 public nonce;

    struct Message {
        address target;
        uint64 nonce;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    function submit(
        Message[] calldata _messages,
        bytes32 _commitment,
        bytes32 _parachainMerkleLeaf,
        uint256 _parachainMerkleLeafIndex,
        uint256 _parachainMerkleLeafCount,
        bytes32[] memory _parachainMerkleProof
    )
        public
        virtual;
}
