// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./WrappedToken.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

contract DOTApp {
    using ScaleCodec for uint128;

    uint256 public balance;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant UNLOCK_CALL = 0x0e01;

    WrappedToken public token;

    uint256 constant internal POLKADOT_DECIMALS = 10;
    uint256 constant internal GRANULARITY = 10 ** (18 - POLKADOT_DECIMALS);

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(Channel memory _basic, Channel memory _incentivized) {
        address[] memory defaultOperators;
        token = new WrappedToken("Wrapped DOT", "WDOT", defaultOperators);

        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;
    }

    function burn(bytes32 _recipient, uint256 _amount, ChannelId _channelId) public {
        require(
            _channelId == ChannelId.Basic ||
            _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        token.burn(msg.sender, _amount, abi.encodePacked(_recipient));

        OutboundChannel channel = OutboundChannel(channels[_channelId].outbound);

        bytes memory call = encodeCall(msg.sender, _recipient, unwrap(_amount));
        channel.submit(call);
    }

    function mint(bytes32 _sender, address _recipient, uint128 _amount) public {
        // TODO: Ensure message sender is a known inbound channel
        token.mint(_recipient, wrap(_amount), abi.encodePacked(_sender));
    }

    function encodeCall(address _sender, bytes32 _recipient, uint128 _amount)
        private
        pure
        returns (bytes memory)
    {
        return
            abi.encodePacked(
                UNLOCK_CALL,
                _sender,
                byte(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode128()
            );
    }

    /**
        Conversion between native and wrapped DOT.
        - Native DOT has 128 bits of precision and 10 decimal places.
        - Wrapped DOT has 256 bits of precision and 18 decimal places.
    */

    function wrap(uint128 _value) pure internal returns (uint256) {
        return uint256(_value) * GRANULARITY;
    }

    function unwrap(uint256 _value) pure internal returns (uint128) {
        return uint128(_value / GRANULARITY);
    }
}
