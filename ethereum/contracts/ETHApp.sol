// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./Decoder.sol";
import "./Application.sol";
import "./SendChannel.sol";

contract ETHApp is Application {
    using SafeMath for uint256;
    using Decoder for bytes;

    uint64 constant PAYLOAD_LENGTH = 84;
    string constant TARGET_APPLICATION_ID = "eth-app";

    address public bridge;
    uint256 public totalETH;
    address public basicSendChannelAddress;
    address public incentivizedSendChannelAddress;

    event Locked(address _sender, bytes32 _recipient, uint256 _amount);
    event Unlock(bytes _sender, address _recipient, uint256 _amount);

    struct ETHLockedPayload {
        address _sender;
        bytes32 _recipient;
        uint256 _amount;
    }

    constructor(
        address _basicSendChannelAddress,
        address _incentivizedSendChannelAddress
    ) public {
        totalETH = 0;
        basicSendChannelAddress = _basicSendChannelAddress;
        incentivizedSendChannelAddress = _incentivizedSendChannelAddress;
    }

    function register(address _bridge) public override {
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
        SendChannel sendChannel;
        if (incentivized) {
            sendChannel = SendChannel(incentivizedSendChannelAddress);
        } else {
            sendChannel = SendChannel(basicSendChannelAddress);
        }
        sendChannel.send(TARGET_APPLICATION_ID, abi.encode(payload));
    }

    function handle(bytes memory _data) public override {
        require(msg.sender == bridge);
        require(_data.length >= PAYLOAD_LENGTH, "Invalid payload");

        // Decode sender bytes
        bytes memory sender = _data.slice(0, 32);
        // Decode recipient address
        address payable recipient = _data.sliceAddress(32);
        // Decode amount int256
        bytes memory amountBytes = _data.slice(32 + 20, 32);
        uint256 amount = amountBytes.decodeUint256();

        unlockETH(recipient, amount);
        emit Unlock(sender, recipient, amount);
    }

    function unlockETH(address payable _recipient, uint256 _amount) internal {
        require(_amount > 0, "Must unlock a positive amount");
        require(
            totalETH >= _amount,
            "ETH token balances insufficient to fulfill the unlock request"
        );

        totalETH = totalETH.sub(_amount);
        _recipient.transfer(_amount);
    }
}
