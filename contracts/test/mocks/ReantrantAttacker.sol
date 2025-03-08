// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IGatewayV1} from "../../src/v1/IGateway.sol";
import {ParaID} from "../../src/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../../src/v1/MultiAddress.sol";
import {console} from "forge-std/console.sol";

contract ReantrantAttacker {
    address public owner;
    address token;
    IGatewayV1 targetContract;
    uint256 targetValue = 0.9 ether;
    uint256 fee;
    ParaID assetHub = ParaID.wrap(1000);
    uint128 amount = 1;
    uint128 extra = 1;
    MultiAddress recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

    constructor(address _targetAddr, address _token) {
        targetContract = IGatewayV1(_targetAddr);
        owner = msg.sender;
        token = _token;
        fee = targetContract.quoteSendTokenFee(_token, assetHub, 0);
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
        targetContract.sendToken{value: amount + fee + extra}(
            token, assetHub, recipientAddress32, 1, amount
        );
    }
}
