// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;
pragma experimental ABIEncoderV2;

interface IOutboundChannel {
    function submit(bytes32 dest, bytes calldata payload) external;
}
