// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./ChannelAccess.sol";
import "./OutboundChannel.sol";

// BasicOutboundChannel is a basic channel that just sends messages with a nonce.
contract BasicOutboundChannel is OutboundChannel, ChannelAccess, AccessControl {

    // Governance contracts will administer using this role.
    bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");

    uint64 public nonce;

    // Only messages originating from this account will
    // be allowed through the channel.
    address public principal;

    event Message(
        address source,
        uint64 nonce,
        bytes payload
    );

    constructor() {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(
        address _configUpdater,
        address _principal,
        address[] memory defaultOperators
    )
    external onlyRole(DEFAULT_ADMIN_ROLE) {
        // Set initial configuration
        grantRole(CONFIG_UPDATE_ROLE, _configUpdater);
        principal = _principal;
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

    // Update the principal.
    function setPrincipal(address _principal) external onlyRole(CONFIG_UPDATE_ROLE) {
        principal = _principal;
    }

    /**
     * @dev Sends a message across the channel
     *
     * Submission is a privileged action involving two parties: The operator and the origin.
     * Apps (aka operators) need to be authorized by the `origin` to submit messages via this channel.
     *
     * Furthermore, this channel restricts the origin to a single account, that of the principal.
     * In essence this ensures that only the principal account can send messages via this channel.
     *
     * For pre-production testing, the restriction to the principal account can be bypassed by using
     * `setPrincipal` to set the principal to the address 0x0000000000000000000000000000000000000042.
     */
    function submit(address _origin, bytes calldata _payload) external override {
        require(isOperatorFor(msg.sender, _origin), "Caller is unauthorized");
        if (principal != address(0x0000000000000000000000000000000000000042)) {
            require(_origin == principal, "Origin is not an authorized principal");
        }
        nonce = nonce + 1;
        emit Message(msg.sender, nonce, _payload);
    }
}
