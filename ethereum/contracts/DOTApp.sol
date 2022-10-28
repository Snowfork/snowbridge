// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./WDOT.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";
import "./FeeController.sol";

enum ChannelId {Basic, Incentivized}

contract DOTApp is FeeController, AccessControl {
    using ScaleCodec for uint256;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant UNLOCK_CALL = 0x4001;

    WDOT public token;

    bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");
    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    bytes32 public constant CHANNEL_UPGRADE_ROLE =
        keccak256("CHANNEL_UPGRADE_ROLE");

    event Minted(bytes32 sender, address recipient, uint256 amount);
    event Burned(address sender, bytes32 recipient, uint256 amount);

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
        address feeBurner,
        Channel memory _basic,
        Channel memory _incentivized
    ) {
        token = new WDOT();

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
        token.burn(msg.sender, _amount);

        OutboundChannel channel =
            OutboundChannel(channels[_channelId].outbound);

        bytes memory call = encodeCall(msg.sender, _recipient, _amount);
        channel.submit(msg.sender, call);

        emit Burned(msg.sender, _recipient, _amount);
    }

    function mint(
        bytes32 _sender,
        address _recipient,
        uint256 _amount
    ) external onlyRole(INBOUND_CHANNEL_ROLE) {
        token.mint(_recipient, _amount);
        emit Minted(_sender, _recipient, _amount);
    }

    // Incentivized channel calls this to charge (burn) fees
    function handleFee(address feePayer, uint256 _amount) external override onlyRole(FEE_BURNER_ROLE) {
        token.burn(feePayer, _amount);
    }

    function encodeCall(
        address _sender,
        bytes32 _recipient,
        uint256 _amount
    ) private pure returns (bytes memory) {
        return
            bytes.concat(
                UNLOCK_CALL,
                abi.encodePacked(_sender),
                bytes1(0x00), // Encoding recipient as MultiAddress::Id
                _recipient,
                _amount.encode256()
            );
    }
}
