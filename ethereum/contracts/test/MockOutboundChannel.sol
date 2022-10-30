// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../OutboundChannel.sol";

contract MockOutboundChannel is OutboundChannel {
    event Message(address source, bytes data);

    function submit(address, bytes calldata data, uint64) external override {
        emit Message(msg.sender, data);
    }
}
