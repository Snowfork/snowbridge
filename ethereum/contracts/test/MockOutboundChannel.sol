// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "../OutboundChannel.sol";

contract MockOutboundChannel is OutboundChannel {
    function submit(
        address,
        uint32,
        uint64,
        bytes calldata
    ) external override {}
}
