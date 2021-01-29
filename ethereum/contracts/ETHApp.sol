// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./Decoder.sol";
import "./OutboundChannel.sol";
import "./ChannelId.sol";

contract ETHApp {
    using SafeMath for uint256;
    using Decoder for bytes;

    uint64 constant PAYLOAD_LENGTH = 84;

    uint256 public balance;
    address public basicOutboundChannelAddress;
    address public incentivizedOutboundChannelAddress;

    event Locked(address sender, bytes32 recipient, uint256 amount);
    event Unlocked(bytes sender, address recipient, uint256 amount);

    struct OutboundPayload {
        address sender;
        bytes32 recipient;
        uint256 amount;
    }

    constructor(
        address _basicOutboundChannelAddress,
        address _incentivizedOutboundChannelAddress
    ) {
        balance = 0;
        basicOutboundChannelAddress = _basicOutboundChannelAddress;
        incentivizedOutboundChannelAddress = _incentivizedOutboundChannelAddress;
    }

    function lock(bytes32 _recipient, ChannelId _channelId) public payable {
        require(msg.value > 0, "Value of transaction must be positive");

        // Increment locked Ethereum counter by this amount
        balance = balance.add(msg.value);

        emit Locked(msg.sender, _recipient, msg.value);

        OutboundPayload memory payload = OutboundPayload(msg.sender, _recipient, msg.value);

        OutboundChannel sendChannel;
        if (_channelId == ChannelId.Incentivized) {
            sendChannel = OutboundChannel(incentivizedOutboundChannelAddress);
        } else {
            sendChannel = OutboundChannel(basicOutboundChannelAddress);
        }
        sendChannel.submit(abi.encode(payload));
    }

    function handle(bytes memory _data)
        public
    {
        require(_data.length >= PAYLOAD_LENGTH, "Invalid payload");

        // Decode sender bytes
        bytes memory sender = _data.slice(0, 32);
        // Decode recipient address
        address payable recipient = _data.sliceAddress(32);
        // Decode amount int256
        bytes memory amountBytes = _data.slice(32 + 20, 32);
        uint256 amount = amountBytes.decodeUint256();

        unlock(recipient, amount);
        emit Unlocked(sender, recipient, amount);
    }

    function unlock(address payable _recipient, uint256 _amount)
        internal
    {
        require(_amount > 0, "Must unlock a positive amount");
        require(balance >= _amount, "ETH token balances insufficient to fulfill the unlock request");

        balance = balance.sub(_amount);
        _recipient.transfer(_amount);
    }
}
