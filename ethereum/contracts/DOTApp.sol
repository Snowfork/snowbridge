// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./WrappedToken.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";
import "./FeeSource.sol";

enum ChannelId {Basic, Incentivized}

contract DOTApp is FeeSource, AccessControl {
    using ScaleCodec for uint256;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant UNLOCK_CALL = 0x4001;

    WrappedToken public token;

    bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");
    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    bytes32 public constant CHANNEL_UPGRADE_ROLE =
        keccak256("CHANNEL_UPGRADE_ROLE");

    event Upgraded(
        address upgrader,
        Channel basic,
        Channel incentivized
    );

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

        _setupRole(CHANNEL_UPGRADE_ROLE, msg.sender);
        _setRoleAdmin(INBOUND_CHANNEL_ROLE, CHANNEL_UPGRADE_ROLE);
        _setRoleAdmin(CHANNEL_UPGRADE_ROLE, CHANNEL_UPGRADE_ROLE);
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

        OutboundChannel channel =
            OutboundChannel(channels[_channelId].outbound);

        bytes memory call = encodeCall(msg.sender, _recipient, _amount);
        channel.submit(msg.sender, call);
    }

    function mint(
        bytes32 _sender,
        address _recipient,
        uint256 _amount
    ) external onlyRole(INBOUND_CHANNEL_ROLE) {
        token.mint(_recipient, _amount, abi.encodePacked(_sender));
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
