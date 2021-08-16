// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

interface OutboundChannel {
    function submit(address origin, bytes calldata payload) external;
}
