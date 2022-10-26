// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../OutboundChannel.sol";

contract MockOutboundChannel is OutboundChannel {
    event Message(address source, bytes data, uint64 weight);

    function submit(address, bytes calldata data, uint64 weight) external override {
        emit Message(msg.sender, data, weight);
    }
}
