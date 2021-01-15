// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

import "./SendChannel.sol";

// IncentivizedSendChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedSendChannel is SendChannel {
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
