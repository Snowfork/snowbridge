 // SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

// ChannelOut contains methods that all outgoing channels must implement
abstract contract ChannelOut {
    /**
     * @dev Submits a message into the channel
     */
    function submit(string memory targetApplicationId, bytes memory payload) public virtual;

}
