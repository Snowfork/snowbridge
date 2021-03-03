// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./OutboundChannel.sol";
import "./DOTApp.sol";

// IncentivizedOutboundChannel is a channel that sends ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutboundChannel is OutboundChannel, AccessControl {

    bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");

    // Nonce for last submitted message
    uint64 public nonce;

    uint256 public relayFee;

    DOTApp private snowdot;

    event RelayFeeUpdated(uint256 previousFee, uint256 newFee);

    event Message(
        address source,
        uint64  nonce,
        uint128 fee,
        bytes   payload
    );

    constructor() {
        // TODO: Give governance contract this role
        _setupRole(CONFIG_UPDATE_ROLE, msg.sender);
    }

    function setDOTApp(address _address) external {
        require(snowdot == DOTApp(address(0)), "ACCESS_FORBIDDEN");
        snowdot = DOTApp(_address);
    }

    function updateRelayFee(uint256 _amount) external {
        require(hasRole(CONFIG_UPDATE_ROLE, msg.sender), "ACCESS_FORBIDDEN");
        uint256 previousFee = relayFee;
        relayFee = _amount;
        emit RelayFeeUpdated(previousFee, _amount);
    }

    /**
     * @dev Sends a message across the channel
     */
    function submit(address origin, bytes calldata payload) external override {
        // burn the fee and retrieve the amount burned in its native denomination
        uint128 nativeFee = snowdot.burnFee(origin, relayFee);

        nonce = nonce + 1;
        emit Message(msg.sender, nonce, nativeFee, payload);
    }
}
