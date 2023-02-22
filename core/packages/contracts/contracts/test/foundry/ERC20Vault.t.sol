pragma solidity ^0.8.9;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../../ERC20Vault.sol";

import "../SovereignAccountMock.sol";
import "../TestToken.sol";

contract ERC20VaultTest is Test {
    event Deposit(address account, address sender, address token, uint256 amount);
    event Withdraw(address account, address recipient, address token, uint256 amount);

    ERC20Vault private vault;
    TestToken private token;
    SovereignAccountMock private account;

    function setUp() public {
        vault = new ERC20Vault();
        token = new TestToken("Test", "T");
        account = new SovereignAccountMock();

        token.mint(address(account), 1000);
    }

    function testInsufficientBalance() public {
        vm.expectRevert(ERC20Vault.InsufficientBalance.selector);
        vault.withdraw(address(account), address(token), 100);
    }

    function testTokenTransferFailedInsufficientAllowance() public {
        account.approveTokenSpend(address(token), address(vault), 50);
        vm.expectRevert("ERC20: insufficient allowance");
        vault.deposit(address(account), address(token), 100);
    }

    function testDepositSuccessful() public {
        account.approveTokenSpend(address(token), address(vault), 100);

        vm.expectEmit(false, false, false, true);
        emit Deposit(address(this), address(account), address(token), 50);

        vault.deposit(address(account), address(token), 50);

        assertEq(token.balanceOf(address(account)), 950);
        assertEq(token.balanceOf(address(vault)), 50);
        assertEq(token.allowance(address(account), address(vault)), 50);
        assertEq(vault.balances(address(token)), 50);
    }

    function testWithdrawSuccessful() public {
        testDepositSuccessful();

        vm.expectEmit(false, false, false, true);
        emit Withdraw(address(this), address(account), address(token), 25);

        vault.withdraw(address(account), address(token), 25);

        assertEq(token.balanceOf(address(account)), 975);
        assertEq(token.balanceOf(address(vault)), 25);
        assertEq(vault.balances(address(token)), 25);
    }

    function testNonOwnerCannotWithdraw() public {
        vault.transferOwnership(address(account));
        vm.expectRevert("Ownable: caller is not the owner");
        vault.withdraw(address(account), address(token), 25);
    }

    function testNonOwnerCannotDeposit() public {
        vault.transferOwnership(address(account));
        vm.expectRevert("Ownable: caller is not the owner");
        vault.withdraw(address(account), address(token), 25);
    }
}
