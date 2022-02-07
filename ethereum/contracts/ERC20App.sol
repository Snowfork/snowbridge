// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

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
        uint32 _paraId,
        uint128 _fee
    ) public {
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        balances[_token] = balances[_token] + _amount;

        emit Locked(_token, msg.sender, _recipient, _amount, _paraId, _fee);

        OutboundChannel channel = OutboundChannel(
            channels[_channelId].outbound
        );

        if (!tokens[_token]) {
            bytes memory createCall = encodeCreateTokenCall(_token);
            tokens[_token] = true;
            channel.submit(msg.sender, createCall);
        }

        bytes memory call;
        if (_paraId == 0) {
            call = encodeCall(_token, msg.sender, _recipient, _amount);
        } else {
            call = encodeCallWithParaId(_token, msg.sender, _recipient, _amount, _paraId, _fee);
        }

        channel.submit(msg.sender, call);

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

    // SCALE-encode payload
    function encodeCall(
        address _token,
        address _sender,
        bytes32 _recipient,
        uint128 _amount
    ) private pure returns (bytes memory) {
        return bytes.concat(
                MINT_CALL,
                abi.encodePacked(_token),
                abi.encodePacked(_sender),
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode128(),
                bytes1(0x00)
            );
    }

    // SCALE-encode payload with parachain Id
    function encodeCallWithParaId(
        address _token,
        address _sender,
        bytes32 _recipient,
        uint128 _amount,
        uint32 _paraId,
        uint128 _fee
    ) private pure returns (bytes memory) {
        return bytes.concat(
                MINT_CALL,
                abi.encodePacked(_token),
                abi.encodePacked(_sender),
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode128(),
                bytes1(0x01),
                _paraId.encode32(),
                _fee.encode128()
            );
    }

    function encodeCreateTokenCall(
        address _token
    ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                CREATE_CALL,
                _token
            );
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
