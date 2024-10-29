// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {IGateway} from "../interfaces/IGateway.sol";
import {ParaID, MultiAddress} from "../Types.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

contract AutoWETH9 {
    address public immutable GATEWAY_PROXY;
    address payable public immutable WETH_INSTANCE;

    event Wrapped(address indexed dst, uint256 wad);
    event Unwrapped(address indexed src, uint256 wad);

    constructor(address _gatewayProxy, address payable _wethInstance) {
        GATEWAY_PROXY = _gatewayProxy;
        WETH_INSTANCE = _wethInstance;
    }

    // Wrap ETH to WETH and transfer using Snowbridge Gateway
    function sendToken(
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        WETH9(WETH_INSTANCE).deposit{value: amount}();
        WETH9(WETH_INSTANCE).approve(GATEWAY_PROXY, amount);
        emit Wrapped(msg.sender, amount);

        uint256 fee = msg.value - amount;
        IGateway(GATEWAY_PROXY).sendToken{value: fee}(
            WETH_INSTANCE, destinationChain, destinationAddress, destinationFee, amount
        );
    }

    // Unwrap WETH to ETH and transfer to destination
    function transferFrom(address src, address payable dst, uint256 wad) public returns (bool) {
        if (WETH9(WETH_INSTANCE).transferFrom(src, address(this), wad)) {
            WETH9(WETH_INSTANCE).withdraw(wad);
            dst.transfer(wad);
            emit Unwrapped(dst, wad);
            return true;
        } else {
            return false;
        }
    }

    ///////////////////////////////////////////////////////
    // ERC20 Interface
    ///////////////////////////////////////////////////////

    function name() public view returns (string memory) {
        return WETH9(WETH_INSTANCE).name();
    }

    function symbol() public view returns (string memory) {
        return WETH9(WETH_INSTANCE).symbol();
    }

    function decimals() public view returns (uint8) {
        return WETH9(WETH_INSTANCE).decimals();
    }

    function balanceOf(address payable who) public view returns (uint) {
        return who.balance;
    }
    
    function allowance(address payable who, address) public view returns (uint) {
        return who.balance;
    }

    function approve(address, uint) public pure returns (bool) {
        return false;
    }

    function totalSupply() public view returns (uint) {
        return WETH9(WETH_INSTANCE).totalSupply();
    }

    function transfer(address payable dst, uint256 wad) public returns (bool) {
        return transferFrom(msg.sender, dst, wad);
    }
}
