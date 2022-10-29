// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./OutboundChannel.sol";

contract ChannelRegistry {

    struct Channel {
        address inbound;
        address outbound;
    }

    // channel id to channel addresses
    mapping(uint32 => Channel) public channels;

    // reverse lookup
    mapping(address => bool) public validInboundChannels;

    function isInboundChannel(address sender) external view returns (bool) {
        return validInboundChannels[sender];
    }

    function outboundChannelForID(uint32 id) external view returns (OutboundChannel) {
        return OutboundChannel(channels[id].outbound);
    }

    function updateChannel(uint32 id, address inbound, address outbound) external {
        delete validInboundChannels[channels[id].inbound];
        channels[id] = Channel(inbound, outbound);
        validInboundChannels[inbound] = true;
    }
}
