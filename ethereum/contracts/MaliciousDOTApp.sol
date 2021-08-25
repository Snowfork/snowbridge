// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./WrappedToken.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";
import "./FeeSource.sol";

enum ChannelId {
    Basic,
    Incentivized
}

// MaliciousDOTApp is similar to DOTApp, but contains an infinite loop in the mint function, which will consume all the
// gas of the message. MaliciousDOTApp is used in a test which verifies that a message running out of gas will not
// prevent execution of other messages
contract MaliciousDOTApp is FeeSource, AccessControl {
    using ScaleCodec for uint256;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant UNLOCK_CALL = 0x4001;

    WrappedToken public token;

    bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");
    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(
        string memory _name,
        string memory _symbol,
        address feeBurner,
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

        _setupRole(FEE_BURNER_ROLE, feeBurner);
        _setupRole(INBOUND_CHANNEL_ROLE, _basic.inbound);
        _setupRole(INBOUND_CHANNEL_ROLE, _incentivized.inbound);
    }

    function burn(
        bytes32 _recipient,
        uint256 _amount,
        ChannelId _channelId
    ) external {
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );
        token.burn(msg.sender, _amount, abi.encodePacked(_recipient));

        OutboundChannel channel = OutboundChannel(
            channels[_channelId].outbound
        );

        bytes memory call = encodeCall(msg.sender, _recipient, _amount);
        channel.submit(msg.sender, call);
    }

    function mint(
        bytes32,
        address,
        uint256
    ) external pure {
        while (true) {}
    }

    // Incentivized channel calls this to charge (burn) fees
    function burnFee(address feePayer, uint256 _amount) external override onlyRole(FEE_BURNER_ROLE) {
        token.burn(feePayer, _amount, "");
    }

    function encodeCall(
        address _sender,
        bytes32 _recipient,
        uint256 _amount
    ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                UNLOCK_CALL,
                _sender,
                bytes1(0x00), // Encoding recipient as MultiAddress::Id
                _recipient,
                _amount.encode256()
            );
    }
}
