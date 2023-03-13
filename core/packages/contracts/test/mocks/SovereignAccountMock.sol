// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "openzeppelin/token/ERC20/IERC20.sol";

contract SovereignAccountMock {
    function approveTokenSpend(address token, address spender, uint256 amount) public {
        IERC20(token).approve(spender, amount);
    }
}
