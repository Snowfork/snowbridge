// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "./OutboundChannel.sol";
import "./ScaleCodec.sol";
import "./ERC20AppPallet.sol";

enum ChannelId {
    Basic,
    Incentivized
}

contract ERC20App is AccessControl {
    using ScaleCodec for uint128;
    using ScaleCodec for uint32;
    using ScaleCodec for uint8;
    using SafeERC20 for IERC20;

    mapping(address => uint128) public balances;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant MINT_CALL = 0x4201;
    bytes2 constant CREATE_CALL = 0x4202;

    mapping(address => bool) public tokens;

    event Locked(
        address token,
        address sender,
        bytes32 recipient,
        uint128 amount,
        uint32 paraId,
        uint128 fee
    );

    event Unlocked(
        address token,
        bytes32 sender,
        address recipient,
        uint128 amount
    );

    event Upgraded(
        address upgrader,
        Channel basic,
        Channel incentivized
    );

    struct Channel {
        address inbound;
        address outbound;
    }

    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    bytes32 public constant CHANNEL_UPGRADE_ROLE =
        keccak256("CHANNEL_UPGRADE_ROLE");


    constructor(Channel memory _basic, Channel memory _incentivized) {
        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;

        _setupRole(CHANNEL_UPGRADE_ROLE, msg.sender);
        _setRoleAdmin(INBOUND_CHANNEL_ROLE, CHANNEL_UPGRADE_ROLE);
        _setRoleAdmin(CHANNEL_UPGRADE_ROLE, CHANNEL_UPGRADE_ROLE);
        _setupRole(INBOUND_CHANNEL_ROLE, _basic.inbound);
        _setupRole(INBOUND_CHANNEL_ROLE, _incentivized.inbound);
    }

    function lock(
        address _token,
        bytes32 _recipient,
        uint128 _amount,
        ChannelId _channelId,
        uint32 _paraID,
        uint128 _fee
    ) public {
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        balances[_token] = balances[_token] + _amount;

        emit Locked(_token, msg.sender, _recipient, _amount, _paraID, _fee);

        OutboundChannel channel = OutboundChannel(
            channels[_channelId].outbound
        );

        if (!tokens[_token]) {
            (bytes memory createCall,) = ERC20AppPallet.create(_token);
            tokens[_token] = true;
            channel.submit(msg.sender, createCall, 0);
        }

        bytes memory call;
        if (_paraID == 0) {
            (call,) = ERC20AppPallet.mint(_token, msg.sender, _recipient, _amount);
        } else {
            (call,) = ERC20AppPallet.mintAndForward(_token, msg.sender, _recipient, _amount, _paraID, _fee);
        }

        channel.submit(msg.sender, call, 0);

        require(
            IERC20(_token).transferFrom(msg.sender, address(this), _amount),
            "Contract token allowances insufficient to complete this lock request"
        );
    }

    function unlock(
        address _token,
        bytes32 _sender,
        address _recipient,
        uint128 _amount
    ) public onlyRole(INBOUND_CHANNEL_ROLE) {
        require(_amount > 0, "Must unlock a positive amount");
        require(
            _amount <= balances[_token],
            "ERC20 token balances insufficient to fulfill the unlock request"
        );

        balances[_token] = balances[_token] - _amount;
        IERC20(_token).safeTransfer(_recipient, _amount);
        emit Unlocked(_token, _sender, _recipient, _amount);
    }

    function upgrade(
        Channel memory _basic,
        Channel memory _incentivized
    ) external onlyRole(CHANNEL_UPGRADE_ROLE) {
        Channel storage c1 = channels[ChannelId.Basic];
        Channel storage c2 = channels[ChannelId.Incentivized];
        // revoke old channel
        revokeRole(INBOUND_CHANNEL_ROLE, c1.inbound);
        revokeRole(INBOUND_CHANNEL_ROLE, c2.inbound);
        // set new channel
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;
        grantRole(INBOUND_CHANNEL_ROLE, _basic.inbound);
        grantRole(INBOUND_CHANNEL_ROLE, _incentivized.inbound);
        emit Upgraded(msg.sender, c1, c2);
    }
}
