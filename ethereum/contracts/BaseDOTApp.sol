// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./WrappedToken.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

abstract contract BaseDOTApp {
    using ScaleCodec for uint128;

    uint256 public balance;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant UNLOCK_CALL = 0x0e01;

    WrappedToken public token;

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(
        string memory _name,
        string memory _symbol,
        Channel memory _basic,
        Channel memory _incentivized
    ) {
        address[] memory defaultOperators;
        token = new WrappedToken(_name, _symbol, defaultOperators);

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
                byte(0x00), // Encoding recipient as MultiAddress::Id
                _recipient,
                _amount.encode128()
            );
    }

    /*
        Conversion between native and wrapped DOT/KSM/ROC.
        For example:
        - Native DOT has 128 bits of precision and 10 decimal places.
        - Wrapped DOT has 256 bits of precision and 18 decimal places.
    */

    function wrap(uint128 _value) pure internal returns (uint256) {
        // No need for SafeMath.div since granularity() resolves to a
        // compile-time constant that is not zero.
        return uint256(_value) * granularity();
    }

    function unwrap(uint256 _value) pure internal returns (uint128) {
        // No need for SafeMath.div since granularity() resolves to a
        // compile-time constant that is not zero (See DOTAppDecimals10.sol and DOTAppDecimals12.sol)
        return uint128(_value / granularity());
    }

    /**
     * Smallest part of DOT that is not divisible when increasing precision to 18 decimal places.
    */
    function granularity() pure internal virtual returns (uint256);

}
