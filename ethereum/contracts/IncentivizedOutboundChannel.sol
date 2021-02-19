// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./OutboundChannel.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel {

    uint256 gasFee;
    address feeController;

    constructor() {
        nonce = 0;
        gasFee = 0;
    }

    event Message(
        address source,
        uint64 nonce,
        bytes payload,
        uint256 fee
    );

    /**
     * @dev Sends a message across the channel
     */
    function submit(bytes memory payload)
        public
        override
    {
        nonce = nonce + 1;
        emit Message(msg.sender, nonce, payload, msg.value);
    }

    modifier onlyFeeController {
        require(msg.sender == feeController);
        _;
    }

    function setGasFee(uint256 _fee) public onlyFeeController {
        require(_fee > 0, "fee must be positive");
        gasFee = _fee;
    }
}
