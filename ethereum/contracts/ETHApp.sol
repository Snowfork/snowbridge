// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";
import "./RewardSource.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

contract ETHApp is RewardSource, AccessControl {
    using SafeMath for uint256;
    using ScaleCodec for uint256;

    uint256 public balance;

    mapping(ChannelId => Channel) public channels;

    event Locked(address sender, bytes32 recipient, uint256 amount);

    event Unlocked(bytes32 sender, address recipient, uint256 amount);

    bytes2 constant MINT_CALL = 0x4101;

    bytes32 public constant REWARD_ROLE = keccak256("REWARD_ROLE");

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(address rewarder, Channel memory _basic, Channel memory _incentivized) {
        balance = 0;

        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;

        _setupRole(REWARD_ROLE, rewarder);
    }

    function lock(bytes32 _recipient, ChannelId _channelId) public payable {
        require(msg.value > 0, "Value of transaction must be positive");
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        balance = balance.add(msg.value);

        emit Locked(msg.sender, _recipient, msg.value);

        bytes memory call = encodeCall(msg.sender, _recipient, msg.value);

        OutboundChannel channel =
            OutboundChannel(channels[_channelId].outbound);
        channel.submit(msg.sender, call);
    }

    function unlock(
        bytes32 _sender,
        address payable _recipient,
        uint256 _amount
    ) public {
        // TODO: Ensure message sender is a known inbound channel
        require(_amount > 0, "Must unlock a positive amount");
        require(
            balance >= _amount,
            "ETH token balances insufficient to fulfill the unlock request"
        );

        balance = balance.sub(_amount);
        _recipient.transfer(_amount);
        emit Unlocked(_sender, _recipient, _amount);
    }

    // SCALE-encode payload
    function encodeCall(
        address _sender,
        bytes32 _recipient,
        uint256 _amount
    ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                MINT_CALL,
                _sender,
                byte(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode256()
            );
    }

    function reward(address payable _recipient, uint256 _amount) external override {
        require(hasRole(REWARD_ROLE, msg.sender), "Caller is unauthorized");
        _recipient.transfer(_amount);
    }

}
