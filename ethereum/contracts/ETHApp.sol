// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./Decoder.sol";
import "./OutboundChannel.sol";

contract ETHApp {
    using SafeMath for uint256;
    using Decoder for bytes;

    uint64 constant PAYLOAD_LENGTH = 84;
    string constant TARGET_APPLICATION_ID = "eth-app";

    address public bridge;
    uint256 public totalETH;
    address public basicOutboundChannelAddress;
    address public incentivizedOutboundChannelAddress;

    event Locked(address _sender, bytes32 _recipient, uint256 _amount);
    event Unlock(address _recipient, uint256 _amount);

    struct ETHLockedPayload {
        address _sender;
        bytes32 _recipient;
        uint256 _amount;
    }

    constructor(
        address _basicOutboundChannelAddress,
        address _incentivizedOutboundChannelAddress
    ) {
        totalETH = 0;
        basicOutboundChannelAddress = _basicOutboundChannelAddress;
        incentivizedOutboundChannelAddress = _incentivizedOutboundChannelAddress;
    }

    function register(address _bridge) public {
        require(bridge == address(0), "Bridge has already been registered");
        bridge = _bridge;
    }

    function sendETH(bytes32 _recipient, bool incentivized) public payable {
        require(msg.value > 0, "Value of transaction must be positive");

        // Increment locked Ethereum counter by this amount
        totalETH = totalETH.add(msg.value);

        emit Locked(msg.sender, _recipient, msg.value);

        ETHLockedPayload memory payload =
            ETHLockedPayload(msg.sender, _recipient, msg.value);
        OutboundChannel sendChannel;
        if (incentivized) {
            sendChannel = OutboundChannel(incentivizedOutboundChannelAddress);
        } else {
            sendChannel = OutboundChannel(basicOutboundChannelAddress);
        }
        sendChannel.submit(abi.encode(payload));
    }

    function unlockETH(address payable _recipient, uint256 _amount) public {
        require(msg.sender == bridge);
        require(_amount > 0, "Must unlock a positive amount");
        require(
            totalETH >= _amount,
            "ETH token balances insufficient to fulfill the unlock request"
        );

        totalETH = totalETH.sub(_amount);
        _recipient.transfer(_amount);
        emit Unlock(_recipient, _amount);
    }
}
