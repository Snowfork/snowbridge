// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./OutboundChannel.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel {

    constructor() {
        nonce = 0;
    }

    /**
     * @dev Sends a message across the channel
     */
    function submit(bytes memory payload)
        public
        override
    {
        nonce = nonce + 1;
        emit Message(tx.origin, msg.sender, nonce, payload);
    }
}
