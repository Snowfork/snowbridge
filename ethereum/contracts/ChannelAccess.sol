// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

abstract contract ChannelAccess {
    mapping(address => bool) private defaultOperators;
    mapping(address => mapping(address => bool)) private operators;

    event OperatorAuthorized(
        address operator,
        address user
    );

    event OperatorRevoked(
        address operator,
        address user
    );

    // Authorize an operator to submit messages for *all* users.
    function _authorizeDefaultOperator(address operator) internal {
        defaultOperators[operator] = true;
    }

    // Revoke authorization for a default operator.
    function _revokeDefaultOperator(address operator) internal {
        require(msg.sender != operator, "Authorizing self as operator");

        delete defaultOperators[operator];
        emit OperatorRevoked(operator, msg.sender);
    }

    // Authorize an operator for the caller.
    function authorizeOperator(address operator) external {
        require(msg.sender != operator, "Authorizing self as operator");

        operators[msg.sender][operator] = true;
        emit OperatorAuthorized(operator, msg.sender);
    }

    // Revoke an operator for the caller.
    function revokeOperator(address operator) external {
        require(msg.sender != operator, "Authorizing self as operator");

        delete operators[msg.sender][operator];
        emit OperatorRevoked(operator, msg.sender);
    }

    function isOperatorFor(address _operator, address _origin) public view returns (bool) {
        return _operator == _origin || defaultOperators[_operator] || operators[_origin][_operator];
    }
}
