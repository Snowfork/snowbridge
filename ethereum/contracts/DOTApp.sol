// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./WrappedToken.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";
import "./FeeSource.sol";

enum ChannelId {Basic, Incentivized}

contract DOTApp is FeeSource {
    using ScaleCodec for uint256;

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

    function burn(bytes32 _recipient, uint256 _amount, ChannelId _channelId) external {
        require(
            _channelId == ChannelId.Basic ||
            _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );
        token.burn(msg.sender, _amount, abi.encodePacked(_recipient));

        OutboundChannel channel = OutboundChannel(channels[_channelId].outbound);

        bytes memory call = encodeCall(msg.sender, _recipient, _amount);
        channel.submit(msg.sender, call);
    }

    function mint(bytes32 _sender, address _recipient, uint256 _amount) external {
        // TODO: Ensure message sender is a known inbound channel
        token.mint(_recipient, _amount, abi.encodePacked(_sender));
    }

    function burnFee(address feePayer, uint256 _amount) external override {
        token.burn(feePayer, _amount, "");
    }

    function encodeCall(address _sender, bytes32 _recipient, uint256 _amount)
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
                _amount.encode256()
            );
    }

}
