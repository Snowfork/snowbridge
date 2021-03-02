// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./OutboundChannel.sol";
import "./DOTApp.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel {

    uint256 public relayFee;
    address public feeController;
    DOTApp private dotApp;

    constructor() {
        nonce = 0;
        relayFee = 0;
        feeController = msg.sender;
    }

    function setDOTApp(address _address) external {
        // TODO: 1. Limit access to owner, 2. Should not reset if set
        dotApp = DOTApp(_address);
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
        // Temporarily commenting burnFee until collective governance is introduced.
        // dotApp.burnFee(user, relayFee);
        nonce = nonce + 1;

        emit Message(msg.sender, nonce, payload, relayFee);
    }

    modifier onlyFeeController {
        require(msg.sender == feeController, "Caller is not a fee controller");
        _;
    }

    /**
    * @dev Sets relayFee. Only feeController is allowed to set relayFee
    */
    function setRelayFee(uint256 _fee) public onlyFeeController {
        require(_fee > 0, "fee must be positive");
        relayFee = _fee;
    }

    /**
    * @dev Change feeController address
    */
    function setFeeController(address _feeControllerAddress) public onlyFeeController {
        feeController = _feeControllerAddress;
    }
}
