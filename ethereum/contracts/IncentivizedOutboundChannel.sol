// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./OutboundChannel.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel {

    // Nonce for last submitted message
    uint64 public nonce;

    event Message(
        address source,
        uint64  nonce,
        bytes   payload
    );

    /**
     * @dev Sends a message across the channel
     */
    function submit(address, bytes calldata payload) external override {
        nonce = nonce + 1;
        emit Message(msg.sender, nonce, payload);
    }
}
