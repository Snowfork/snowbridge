// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./OutboundChannel.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ChannelRegistry is Ownable {
    struct Channel {
        address inbound;
        address outbound;
    }

    // channel id to channel addresses
    mapping(uint32 => Channel) public channels;

    // valid inbound channels
    mapping(address => bool) public validInboundChannels;

    event ChannelUpdated(uint32 id, address inbound, address outbound);
    event ChannelRemoved(uint32 id);

    // Check to see that sender is a valid inbound channel
    function isInboundChannel(address sender) external view returns (bool) {
        return validInboundChannels[sender];
    }

    // Fetch address of outbound channel identified by `id`
    function outboundChannelForID(uint32 id) external view returns (address) {
        return channels[id].outbound;
    }

    function updateChannel(uint32 id, address inbound, address outbound) external onlyOwner {
        delete validInboundChannels[channels[id].inbound];
        channels[id] = Channel(inbound, outbound);
        validInboundChannels[inbound] = true;
        emit ChannelUpdated(id, inbound, outbound);
    }

    function removeChannel(uint32 id) external onlyOwner {
        delete validInboundChannels[channels[id].inbound];
        delete channels[id];
        emit ChannelRemoved(id);
    }
}
