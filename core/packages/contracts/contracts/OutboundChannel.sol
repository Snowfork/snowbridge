// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;
pragma experimental ABIEncoderV2;

interface OutboundChannel {
    function submit(address account, bytes calldata payload, uint64 weight) external;
}
