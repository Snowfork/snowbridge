// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

abstract contract InboundChannel {

    uint64 public nonce;

    struct Message {
        address target;
        uint256 nonce;
        bytes payload;
    }

    event MessageDelivered(uint256 nonce, bool result);

    function submit(Message[] memory commitment) virtual public;
}
