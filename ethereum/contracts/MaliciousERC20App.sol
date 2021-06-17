// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

contract MaliciousERC20App {
    using SafeMath for uint256;
    using ScaleCodec for uint256;

    mapping(address => uint256) public balances;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant MINT_CALL = 0x4201;

    event Locked(
        address token,
        address sender,
        bytes32 recipient,
        uint256 amount
    );

    event Unlocked(
        address token,
        bytes32 sender,
        address recipient,
        uint256 amount
    );

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(Channel memory _basic, Channel memory _incentivized) {
        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;
    }

    function lock(
        address _token,
        bytes32 _recipient,
        uint256 _amount,
        ChannelId _channelId
    ) public {
        require(
            IERC20(_token).transferFrom(msg.sender, address(this), _amount),
            "Contract token allowances insufficient to complete this lock request"
        );
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        balances[_token] = balances[_token].add(_amount);

        emit Locked(_token, msg.sender, _recipient, _amount);

        bytes memory call = encodeCall(_token, msg.sender, _recipient, _amount);

        OutboundChannel channel =
            OutboundChannel(channels[_channelId].outbound);
        channel.submit(msg.sender, call);
    }

    function unlock(
        address _token,
        bytes32 _sender,
        address _recipient,
        uint256 _amount
    ) public pure {
        while (true) {}
    }

    // SCALE-encode payload
    function encodeCall(
        address _token,
        address _sender,
        bytes32 _recipient,
        uint256 _amount
    ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                MINT_CALL,
                _token,
                _sender,
                byte(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode256()
            );
    }
}
