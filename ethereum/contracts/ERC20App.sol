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
    using ScaleCodec for uint256;
    using ScaleCodec for uint32;
    using ScaleCodec for uint8;
    using SafeERC20 for IERC20;

    mapping(address => uint256) public balances;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant MINT_CALL = 0x4201;

    bytes2 constant CREATE_CALL = 0x4202;

    mapping(address => bool) public wrappedTokenList;

    event Locked(
        address token,
        address sender,
        bytes32 recipient,
        uint256 amount,
        uint32 paraId
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

    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    constructor(Channel memory _basic, Channel memory _incentivized) {
        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;

        _setupRole(INBOUND_CHANNEL_ROLE, _basic.inbound);
        _setupRole(INBOUND_CHANNEL_ROLE, _incentivized.inbound);
    }

    function lock(
        address _token,
        bytes32 _recipient,
        uint256 _amount,
        ChannelId _channelId,
        uint32 _paraId
    ) public {
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        balances[_token] = balances[_token] + _amount;

        emit Locked(_token, msg.sender, _recipient, _amount, _paraId);

        OutboundChannel channel = OutboundChannel(
            channels[_channelId].outbound
        );

        if (!wrappedTokenList[_token]) {
            bytes memory createCall = encodeToken(_token);
            wrappedTokenList[_token] = true;
            channel.submit(msg.sender, createCall);
        }

        bytes memory call;
        if(_paraId == 0) {
            call = encodeCall(_token, msg.sender, _recipient, _amount);
        } else {
            call = encodeCallWithParaId(_token, msg.sender, _recipient, _amount, _paraId);
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
        uint256 _amount
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
        uint256 _amount
    ) private pure returns (bytes memory) {
        return abi.encodePacked(
                MINT_CALL,
                _token,
                _sender,
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode256(),
                bytes1(0x00)
            );
    }

    // SCALE-encode payload with parachain Id
    function encodeCallWithParaId(
        address _token,
        address _sender,
        bytes32 _recipient,
        uint256 _amount,
        uint32 _paraId
    ) private pure returns (bytes memory) {
        return abi.encodePacked(
                MINT_CALL,
                _token,
                _sender,
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode256(),
                bytes1(0x01),
                _paraId.encode32()
            );
    }

    function encodeCreateTokenCall(
        address _token,
        string memory _name,
        string memory _symbol,
        uint8 _decimals
    ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                CREATE_CALL,
                _token,
                _name,
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _symbol,
                _decimals.encode8()
            );
    }

    function tokenDetails(address _token)
        private
        view
        returns (
            string memory,
            string memory,
            uint8
        )
    {
        ERC20 metadata = ERC20(_token);

        uint8 _decimals;
        string memory _name;
        string memory _symbol;

        try metadata.name() returns (string memory name) {
            _name = name;
        } catch {
            _name = "";
        }

        try metadata.symbol() returns (string memory symbol) {
            _symbol = symbol;
        } catch {
            _symbol = "";
        }

        try metadata.decimals() returns (uint8 decimal) {
            _decimals = decimal;
        } catch {
            _decimals = 0;
        }
        return (_name, _symbol, _decimals);
    }

    function encodeToken(address _token) private view returns (bytes memory) {
        uint8 _decimals;
        string memory _name;
        string memory _symbol;

        (_name, _symbol, _decimals) = tokenDetails(_token);
        bytes memory createCall;
        createCall = encodeCreateTokenCall(_token, _name, _symbol, _decimals);
        return (createCall);
    }
}
