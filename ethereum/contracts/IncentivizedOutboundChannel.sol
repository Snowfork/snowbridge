// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./OutboundChannel.sol";
import "./FeeSource.sol"

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel {

    // Nonce for last submitted message
    uint64 public nonce;

    uint256 private fee;
    FeeSource private feeSource;

    mapping(address => bool) private defaultOperators;
    mapping(address => mapping(address => bool)) private operators;
    mapping(address => mapping(address => bool)) private revokedDefaultOperators;

    event Message(
        address source,
        uint64  nonce,
        uint256 fee,
        bytes   payload
    );

    event OperatorAuthorized(
        address operator,
        address feePayer
    );

    event FeeChanged(
        uint256 oldFee,
        uint256 newFee
    );

    function setFeeSource(address _feeSource) external {
        feeSource = FeeSource(_feeSource);
    }

    function setFee(uint256 _amount) external {
        let _oldFee = fee;
        fee = _amount;
        emit FeeChanged(oldFee, _amount);
    }

    function addDefaultOperator(address app) external {
        defaultOperators[app] = true;
    }

    function authorizeOperator(address operator) external {
        require(msg.sender != operator, "Authorizing self as operator");

        if (defaultOperators[operator]) {
            delete revokedDefaultOperators[msg.sender][operator];
        } else {
            operators[msg.sender][operator] = true;
        }

        emit OperatorAuthorized(operator, msg.sender);
    }

    function isOperatorFor(address _operator, address _origin) public view returns (bool) {
        return _operator == _origin ||
            (defaultOperators[_operator] && !revokedDefaultOperators[_origin][_operator]) ||
            operators[_origin][_operator];
    }

    /**
     * @dev Sends a message across the channel
     */
    function submit(address feePayer, bytes calldata payload) external override {
        require(isOperatorFor(msg.sender, feePayer), "Caller is not an operator for fee payer");
        feeSource.burnFee(feePayer, fee);
        nonce = nonce + 1;
        emit Message(msg.sender, nonce, fee, payload);
    }
}
