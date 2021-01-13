 // SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

// OutChannel contains methods that all outgoing channels must implement
abstract contract OutChannel {

    event NewMessage(uint256 nonce, address senderAddress, string targetApplicationId, bytes payload);

    /**
     * @dev Submits a message to the channel
     */
    function submit(string memory targetApplicationId, bytes memory payload) public virtual;

}
