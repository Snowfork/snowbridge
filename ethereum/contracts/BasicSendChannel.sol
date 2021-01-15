// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

import "./SendChannel.sol";

// BasicSendChannel is a basic channel that just sends messages with a nonce.
contract BasicSendChannel is SendChannel {
    uint256 public currentNonce;

    constructor() public {
        currentNonce = 0;
    }

    /**
     * @dev Sends a message across the channel
     */
    function send(string memory targetApplicationId, bytes memory payload)
        public
        override
    {
        emit NewMessage(currentNonce, msg.sender, targetApplicationId, payload);
        currentNonce++;
    }
}
