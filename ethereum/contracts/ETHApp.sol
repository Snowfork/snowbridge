// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/math/SafeCast.sol";
import "./RewardController.sol";
import "./OutboundChannel.sol";
import "./ETHAppPallet.sol";
import "./ChannelRegistry.sol";

enum ChannelId {
    Basic,
    Incentivized
}

contract ETHApp is RewardController, AccessControl {
    using SafeCast for uint256;

    ChannelRegistry public immutable registry;

    bytes32 public constant REWARD_ROLE = keccak256("REWARD_ROLE");


    event Locked(
        address sender,
        bytes32 recipient,
        uint128 amount,
        uint32 paraId,
        uint128 fee
    );

    event Unlocked(bytes32 sender, address recipient, uint128 amount);

    // Unknown outbound channel
    error UnknownChannel(uint32 id);

    // Not allowed to send messages to this app
    error Unauthorized();

    // Value of transaction must be positive
    error MinimumAmount();

    // Cannot send ether to recipient
    error CannotUnlock();

    constructor(
        address rewarder,
        address channelRegistry
    ) {
        registry = ChannelRegistry(channelRegistry);
        _setupRole(REWARD_ROLE, rewarder);
    }

    function lock(
        bytes32 _recipient,
        uint32 _paraID,
        uint128 _fee,
        uint32 _channelID
    ) public payable {
        if (msg.value == 0) {
            revert MinimumAmount();
        }

        address channel = registry.outboundChannelForID(_channelID);
        if (channel == address(0)) {
            revert UnknownChannel(_channelID);
        }

        // revert in case of overflow.
        uint128 value = (msg.value).toUint128();

        emit Locked(msg.sender, _recipient, value, _paraID, _fee);

        bytes memory call;
        uint64 weight;
        if (_paraID == 0) {
            (call, weight) = ETHAppPallet.mint(msg.sender, _recipient, value);
        } else {
            (call, weight) = ETHAppPallet.mintAndForward(msg.sender, _recipient, value, _paraID, _fee);
        }

        OutboundChannel(channel).submit(msg.sender, call, weight);
    }

    function unlock(
        bytes32 _sender,
        address payable _recipient,
        uint128 _amount
    ) external {
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }

        (bool success, ) = _recipient.call{value: _amount}("");
        if (!success) {
            revert CannotUnlock();
        }

        emit Unlocked(_sender, _recipient, _amount);
    }

    function handleReward(address payable _relayer, uint128 _amount)
        external
        override
        onlyRole(REWARD_ROLE)
    {
        (bool success,) = _relayer.call{value: _amount}("");
        if (success) {
            emit Rewarded(_relayer, _amount);
        }
    }
}
