// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {IERC20} from "../src/interfaces/IERC20.sol";
import {Token} from "../src/Token.sol";
import {TokenLib} from "../src/TokenLib.sol";

// See https://mirror.xyz/horsefacts.eth/Jex2YVaO65dda6zEyfM_-DXlXhOWCAoSpOx5PLocYgw for invariant testing

contract TokenTest is Test {
    function setUp() public {}

    function testFuzz_metadata(string memory name, string memory symbol, uint8 decimals) public {
        Token token = new Token(name, symbol, decimals);

        assertEq(token.name(), name);
        assertEq(token.symbol(), symbol);
        assertEq(token.decimals(), decimals);
    }

    function testFuzz_mint(address account, uint256 amount) public {
        vm.assume(account != address(0));

        Token token = new Token("", "", 18);

        token.mint(account, amount);

        assertEq(token.totalSupply(), amount);
        assertEq(token.balanceOf(account), amount);
    }

    function testFuzz_burn(address account, uint256 balance, uint256 burnAmount) public {
        vm.assume(account != address(0));
        vm.assume(balance >= burnAmount);

        Token token = new Token("", "", 18);

        token.mint(account, balance);
        token.burn(account, burnAmount);

        assertEq(token.totalSupply(), balance - burnAmount);
        assertEq(token.balanceOf(account), balance - burnAmount);
    }

    function testFuzz_approve(address account, uint256 amount) public {
        vm.assume(account != address(0));

        Token token = new Token("", "", 18);

        assertTrue(token.approve(account, amount));
        assertEq(token.allowance(address(this), account), amount);
    }

    function testFuzz_increaseAllowance(address account, uint256 initialAmount, uint256 addedAmount) public {
        vm.assume(account != address(0));
        vm.assume(initialAmount <= type(uint256).max / 2);
        vm.assume(addedAmount <= type(uint256).max / 2);

        Token token = new Token("", "", 18);

        token.approve(account, initialAmount);

        assertEq(token.allowance(address(this), account), initialAmount);
        assertTrue(token.increaseAllowance(account, addedAmount));
        assertEq(token.allowance(address(this), account), initialAmount + addedAmount);
    }

    function testFuzz_decreaseAllowance(address account, uint256 initialAmount, uint256 subtractedAmount) public {
        vm.assume(account != address(0));
        vm.assume(initialAmount > 0 && subtractedAmount > 0 && initialAmount >= subtractedAmount);

        Token token = new Token("", "", 18);

        token.approve(account, initialAmount);

        assertEq(token.allowance(address(this), account), initialAmount);
        assertTrue(token.decreaseAllowance(account, subtractedAmount));
        assertEq(token.allowance(address(this), account), initialAmount - subtractedAmount);
    }

    function testFuzz_transferCorrectlyUpdatesBalances(
        address sender,
        address receiver,
        uint256 mintAmount,
        uint256 transferAmount
    ) public {
        vm.assume(sender != address(0) && receiver != address(0) && sender != receiver);
        vm.assume(mintAmount > 0 && transferAmount > 0 && mintAmount >= transferAmount);

        Token token = new Token("", "", 18);

        token.mint(sender, mintAmount);

        uint256 initialSenderBalance = token.balanceOf(sender);
        uint256 initialReceiverBalance = token.balanceOf(receiver);
        uint256 initialTotalSupply = token.totalSupply();

        vm.prank(sender);
        token.transfer(receiver, transferAmount);

        assertEq(token.balanceOf(sender), initialSenderBalance - transferAmount);
        assertEq(token.balanceOf(receiver), initialReceiverBalance + transferAmount);
        assertEq(token.totalSupply(), initialTotalSupply);
    }

    function testFuzz_transferFrom(
        address owner,
        address spender,
        address receiver,
        uint256 mintAmount,
        uint256 allowanceAmount,
        uint256 transferAmount
    ) public {
        // Avoid zero address and ensure distinct addresses
        vm.assume(owner != address(0) && spender != address(0) && receiver != address(0));
        vm.assume(owner != spender && spender != receiver && owner != receiver);

        // Ensure amounts are valid
        vm.assume(mintAmount > 0 && allowanceAmount > 0 && transferAmount > 0);
        vm.assume(mintAmount >= transferAmount);
        vm.assume(allowanceAmount >= transferAmount);
        vm.assume(allowanceAmount < type(uint256).max);

        Token token = new Token("", "", 18);

        // Mint tokens to owner
        token.mint(owner, mintAmount);

        // Set allowance for spender
        vm.prank(owner);
        token.approve(spender, allowanceAmount);

        uint256 initialOwnerBalance = token.balanceOf(owner);
        uint256 initialReceiverBalance = token.balanceOf(receiver);
        uint256 initialAllowance = token.allowance(owner, spender);
        uint256 initialTotalSupply = token.totalSupply();

        // Perform transferFrom as spender
        vm.prank(spender);
        token.transferFrom(owner, receiver, transferAmount);

        // Verify balances are updated correctly
        assertEq(token.balanceOf(owner), initialOwnerBalance - transferAmount);
        assertEq(token.balanceOf(receiver), initialReceiverBalance + transferAmount);

        // Verify allowance is reduced
        assertEq(token.allowance(owner, spender), initialAllowance - transferAmount);

        // Verify total supply remains unchanged
        assertEq(token.totalSupply(), initialTotalSupply);
    }

    // This additional test specifically verifies the behavior when
    // using maximum allowance (`type(uint256).max`), which is a special case where the allowance
    // should not be reduced after the transfer.
    function testFuzz_transferFromWithMaxAllowance(
        address owner,
        address spender,
        address receiver,
        uint256 mintAmount,
        uint256 transferAmount
    ) public {
        vm.assume(owner != address(0) && spender != address(0) && receiver != address(0));
        vm.assume(owner != spender && spender != receiver && owner != receiver);
        vm.assume(mintAmount > 0 && transferAmount > 0);
        vm.assume(mintAmount >= transferAmount);

        Token token = new Token("", "", 18);

        token.mint(owner, mintAmount);

        // Approve maximum allowance
        vm.prank(owner);
        token.approve(spender, type(uint256).max);

        uint256 initialOwnerBalance = token.balanceOf(owner);
        uint256 initialReceiverBalance = token.balanceOf(receiver);

        vm.prank(spender);
        token.transferFrom(owner, receiver, transferAmount);

        // Verify balances
        assertEq(token.balanceOf(owner), initialOwnerBalance - transferAmount);
        assertEq(token.balanceOf(receiver), initialReceiverBalance + transferAmount);

        // Verify allowance remains unchanged for max approval
        assertEq(token.allowance(owner, spender), type(uint256).max);
    }

    function test_transferFromToZeroAddressReverts() public {
        Token token = new Token("Test", "TST", 18);

        address owner = makeAddr("owner");
        address spender = makeAddr("spender");
        uint256 amount = 100;

        // Mint tokens to owner
        token.mint(owner, amount);

        // Approve spender
        vm.prank(owner);
        token.approve(spender, amount);

        // Attempt transfer to zero address should revert
        vm.prank(spender);
        vm.expectRevert(IERC20.InvalidAccount.selector);
        token.transferFrom(owner, address(0), amount);
    }
}
