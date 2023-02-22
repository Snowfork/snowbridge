// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../OutboundChannel.sol";

contract OutboundChannelMock is OutboundChannel {
    event Message(address account, bytes payload, uint64 weight);

    function submit(address account, bytes calldata payload, uint64 weight) external {
        emit Message(account, payload, weight);
    }
}
