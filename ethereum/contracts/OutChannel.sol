 // SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

// OutChannel contains methods that all outgoing channels must implement
abstract contract OutChannel {
    /**
     * @dev Submits a message to the channel
     */
    function submit(string memory targetApplicationId, bytes memory payload) public virtual;

}
