// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./RewardSource.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {
    Basic,
    Incentivized
}

contract ETHApp is RewardSource, AccessControl {
    using ScaleCodec for uint256;
    using ScaleCodec for uint32;

    mapping(ChannelId => Channel) public channels;

    event Locked(
        address sender,
        bytes32 recipient,
        uint256 amount,
        uint32 paraId
    );

    event Unlocked(bytes32 sender, address recipient, uint256 amount);

    event Upgraded(
        address upgrader,
        Channel basic,
        Channel incentivized
    );

    bytes2 constant MINT_CALL = 0x4101;

    bytes32 public constant REWARD_ROLE = keccak256("REWARD_ROLE");

    struct Channel {
        address inbound;
        address outbound;
    }

    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    bytes32 public constant CHANNEL_UPGRADE_ROLE =
        keccak256("CHANNEL_UPGRADE_ROLE");

    constructor(
        address rewarder,
        Channel memory _basic,
        Channel memory _incentivized
    ) {

        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;

        _setupRole(CHANNEL_UPGRADE_ROLE, msg.sender);
        _setRoleAdmin(INBOUND_CHANNEL_ROLE, CHANNEL_UPGRADE_ROLE);
        _setRoleAdmin(CHANNEL_UPGRADE_ROLE, CHANNEL_UPGRADE_ROLE);
        _setupRole(REWARD_ROLE, rewarder);
        _setupRole(INBOUND_CHANNEL_ROLE, _basic.inbound);
        _setupRole(INBOUND_CHANNEL_ROLE, _incentivized.inbound);
    }

    function lock(
        bytes32 _recipient,
        ChannelId _channelId,
        uint32 _paraId
    ) public payable {
        require(msg.value > 0, "Value of transaction must be positive");
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        emit Locked(msg.sender, _recipient, msg.value, _paraId);

        bytes memory call;
        if(_paraId == 0) {
            call = encodeCall(
                msg.sender,
                _recipient,
                msg.value
            );
        } else {
            call = encodeCallWithParaId(
                msg.sender,
                _recipient,
                msg.value,
                _paraId
            );
        }

        OutboundChannel channel = OutboundChannel(
            channels[_channelId].outbound
        );
        channel.submit(msg.sender, call);
    }

    function unlock(
        bytes32 _sender,
        address payable _recipient,
        uint256 _amount
    ) public onlyRole(INBOUND_CHANNEL_ROLE) {
        require(_amount > 0, "Must unlock a positive amount");

        (bool success, ) = _recipient.call{value: _amount}("");
        require(success, "Unable to send Ether");
        emit Unlocked(_sender, _recipient, _amount);
    }

    // SCALE-encode payload
    function encodeCall(
        address _sender,
        bytes32 _recipient,
        uint256 _amount
    ) private pure returns (bytes memory) {
        return abi.encodePacked(
                MINT_CALL,
                _sender,
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode256(),
                bytes1(0x00)
            );
    }

    // SCALE-encode payload with parachain Id
    function encodeCallWithParaId(
        address _sender,
        bytes32 _recipient,
        uint256 _amount,
        uint32 _paraId
    ) private pure returns (bytes memory) {
        return abi.encodePacked(
                MINT_CALL,
                _sender,
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode256(),
                bytes1(0x01),
                _paraId.encode32()
            );
    }

    function reward(address payable _recipient, uint256 _amount)
        external
        override
        onlyRole(REWARD_ROLE)
    {
        (bool success, ) = _recipient.call{value: _amount}("");
        require(success, "Unable to send Ether");
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
