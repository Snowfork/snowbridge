// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import "openzeppelin/token/ERC20/IERC20.sol";

import "../src/TokenVault.sol";

contract TokenVaultTest is Test {
    event Deposit(address sender, address token, uint256 amount);
    event Withdraw(address recipient, address token, uint256 amount);

    TokenVault vault;
    WETH9 token;
    address account;

    function setUp() public {
        vault = new TokenVault();
        token = new WETH9();

        account = makeAddr("statemint");

        vault.grantRole(vault.WITHDRAW_ROLE(), address(this));
        vault.grantRole(vault.DEPOSIT_ROLE(), address(this));

        // create tokens for statemint account
        hoax(account);
        token.deposit{value: 1000}();
    }

    function testInsufficientBalance() public {
        vm.expectRevert(TokenVault.InsufficientBalance.selector);
        vault.withdraw(address(account), address(token), 100);
    }

    function testTokenTransferFailedInsufficientAllowance() public {
        hoax(account);
        token.approve(address(vault), 50);

        vm.expectRevert();
        vault.deposit(address(account), address(token), 100);
    }

    function testDepositSuccessful() public {
        hoax(account);
        token.approve(address(vault), 100);

        vm.expectEmit(false, false, false, true);
        emit Deposit(address(account), address(token), 50);

        vault.deposit(address(account), address(token), 50);

        assertEq(token.balanceOf(address(account)), 950);
        assertEq(token.balanceOf(address(vault)), 50);
        assertEq(token.allowance(address(account), address(vault)), 50);
        assertEq(vault.balance(address(token)), 50);
    }

    function testWithdrawSuccessful() public {
        testDepositSuccessful();

        vm.expectEmit(false, false, false, true);
        emit Withdraw(address(account), address(token), 25);

        vault.withdraw(address(account), address(token), 25);

        assertEq(token.balanceOf(address(account)), 975);
        assertEq(token.balanceOf(address(vault)), 25);
        assertEq(vault.balance(address(token)), 25);
    }

    function testNonOwnerCannotWithdraw() public {
        vault.revokeRole(vault.WITHDRAW_ROLE(), address(this));
        vm.expectRevert(
            "AccessControl: account 0x7fa9385be102ac3eac297483dd6233d62b3e1496 is missing role 0x5d8e12c39142ff96d79d04d15d1ba1269e4fe57bb9d26f43523628b34ba108ec"
        );
        vault.withdraw(address(account), address(token), 25);
    }
}
