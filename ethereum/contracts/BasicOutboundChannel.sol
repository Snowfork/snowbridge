// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./ChannelAccess.sol";
import "./OutboundChannel.sol";

// BasicOutboundChannel is a basic channel that just sends messages with a nonce.
contract BasicOutboundChannel is OutboundChannel, ChannelAccess, AccessControl {

    // Governance contracts will administer using this role.
    bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");

    mapping(address => uint64) public nonce;

    event Message(
        address source,
        address account,
        uint64 nonce,
        bytes payload,
        uint64 weight
    );

    constructor() {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(
        address _configUpdater,
        address[] memory defaultOperators
    )
    external onlyRole(DEFAULT_ADMIN_ROLE) {
        // Set initial configuration
        grantRole(CONFIG_UPDATE_ROLE, _configUpdater);
        for (uint i = 0; i < defaultOperators.length; i++) {
            _authorizeDefaultOperator(defaultOperators[i]);
        }

        // drop admin privileges
        renounceRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // Authorize an operator/app to submit messages for *all* users.
    function authorizeDefaultOperator(address operator) external onlyRole(CONFIG_UPDATE_ROLE) {
        _authorizeDefaultOperator(operator);
    }

    // Revoke authorization.
    function revokeDefaultOperator(address operator) external onlyRole(CONFIG_UPDATE_ROLE) {
        _revokeDefaultOperator(operator);
    }

    /**
     * @dev Sends a message across the channel
     *
     * Submission is a privileged action involving two parties: The operator and the origin (called account here).
     * Apps (aka operators) need to be authorized by the `account` to submit messages via this channel.
     */
    function submit(address _account, bytes calldata _payload, uint64 weight) external override {
        require(isOperatorFor(msg.sender, _account), "Caller is unauthorized");
        nonce[_account] = nonce[_account] + 1;
        emit Message(msg.sender, _account, nonce[_account], _payload, weight);
    }
}
