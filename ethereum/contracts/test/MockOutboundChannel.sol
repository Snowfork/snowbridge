// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "../OutboundChannel.sol";

contract MockOutboundChannel is OutboundChannel {
    event Message(address source, bytes data);

    function submit(address, bytes calldata data) external override {
        emit Message(msg.sender, data);
    }
}
