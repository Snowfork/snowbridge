// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./OutboundChannel.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel {

    uint256 private _relayFee;
    address private _feeController;

    function relayFee() public view virtual returns (uint256) {
        return _relayFee;
    }

      function feeController() public view virtual returns (address) {
        return _feeController;
    }

    constructor(uint256 _fee, address _feeControllerAddress) {
        nonce = 0;
        _relayFee = _fee;
        _feeController = _feeControllerAddress;
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


        emit Message(msg.sender, nonce, payload, _relayFee);
    }

    modifier onlyFeeController {
        require(msg.sender == _feeController, "Caller is not a fee controller");
        _;
    }

    /**
    * @dev Sets relayFee. Only feeController is allowed to set relayFee
    */
    function setRelayFee(uint256 _fee) public onlyFeeController {
        require(_fee > 0, "fee must be positive");
        _relayFee = _fee;
    }

    /**
    * @dev Change feeController address
    */
    function setFeeController(address _feeControllerAddress) public onlyFeeController {
        _feeController = _feeControllerAddress;
    }
}
