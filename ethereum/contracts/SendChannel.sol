// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

// SendChannel contains methods that all outgoing channels must implement
abstract contract SendChannel {
    event NewMessage(
        uint256 nonce,
        address senderAddress,
        string targetApplicationId,
        bytes payload
    );

    /**
     * @dev Sends a message across the channel
     */
    function send(string memory targetApplicationId, bytes memory payload)
        public
        virtual;
}
