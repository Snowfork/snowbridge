// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./OutboundChannel.sol";
import "./ChannelAccess.sol";
import "./FeeSource.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have
// incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel, ChannelAccess, AccessControl {

    // Governance contracts will administer using this role.
    bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");

    // Nonce for last submitted message
    uint64 public nonce;

    uint256 public fee;
    FeeSource public feeSource;

    event Message(
        address source,
        uint64  nonce,
        uint256 fee,
        bytes   payload
    );

    event FeeChanged(
        uint256 oldFee,
        uint256 newFee
    );

    constructor() {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // Once-off post-construction call to set initial configuration.
    function initialize(
        address _configUpdater,
        address _feeSource,
        address[] memory defaultOperators
    )
    external onlyRole(DEFAULT_ADMIN_ROLE) {
        // Set initial configuration
        feeSource = FeeSource(_feeSource);
        grantRole(CONFIG_UPDATE_ROLE, _configUpdater);
        for (uint i = 0; i < defaultOperators.length; i++) {
            _authorizeDefaultOperator(defaultOperators[i]);
        }

        // drop admin privileges
        renounceRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    // Update message submission fee.
    function setFee(uint256 _amount) external onlyRole(CONFIG_UPDATE_ROLE) {
        emit FeeChanged(fee, _amount);
        fee = _amount;
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
     */
    function submit(address feePayer, bytes calldata payload) external override {
        require(isOperatorFor(msg.sender, feePayer), "Caller is not an operator for fee payer");
        feeSource.burnFee(feePayer, fee);
        nonce = nonce + 1;
        emit Message(msg.sender, nonce, fee, payload);
    }
}
