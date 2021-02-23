// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./OutboundChannel.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel {

    uint256 public gasFee;
    address public feeController;

    constructor(uint256 _gasFee, address _feeControllerAddress) {
        nonce = 0;
        gasFee = _gasFee;
        feeController = _feeControllerAddress;
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


        emit Message(msg.sender, nonce, payload, gasFee);
    }

    modifier onlyFeeController {
        require(msg.sender == feeController, "Caller is not a fee controller");
        _;
    }

    /**
    * @dev Sets gasFee. Only feeController is allowed to set gasFee
    */
    function setGasFee(uint256 _fee) public onlyFeeController {
        require(_fee > 0, "fee must be positive");
        gasFee = _fee;
    }

    /**
    * @dev Change feeController address
    */
    function setFeeController(address _feeControllerAddress) public onlyFeeController {
        feeController = _feeControllerAddress;
    }
}
