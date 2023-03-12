// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../IOutboundChannel.sol";

contract OutboundChannelMock is IOutboundChannel {
    event Message(bytes dest, bytes payload);

    function submit(bytes calldata dest, bytes calldata payload) external payable {
        emit Message(dest, payload);
    }
}
