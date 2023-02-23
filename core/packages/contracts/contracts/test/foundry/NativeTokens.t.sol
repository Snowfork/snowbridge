// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "forge-std/Test.sol";
import "forge-std/console2.sol";

import "../../NativeTokens.sol";

import "../OutboundChannelMock.sol";
import "../SovereignAccountMock.sol";
import "../TestToken.sol";

contract NativeTokensTest is Test {
    event Deposit(address account, address sender, address token, uint256 amount);
    event Withdraw(address account, address recipient, address token, uint256 amount);
    event Locked(address origin, bytes32 recipient, address token, uint128 amount);
    event Unlocked(bytes32 origin, address recipient, address token, uint128 amount);
    event Created(address token, string name, string symbol, uint8 decimals);
    event Message(address account, bytes payload, uint64 weight);

    ERC20Vault private vault;
    NativeTokens private nativeTokens;
    OutboundChannel private outboundChannel;
    TestToken private token;
    SovereignAccountMock private account1;
    address private account2;

    bytes32 private constant origin = keccak256("STATEMINT");
    bytes32 private constant recipient = keccak256("STATEMINT/0xABCDEF1234567890");

    function setUp() public {
        token = new TestToken("Test", "T");

        outboundChannel = new OutboundChannelMock();
        vault = new ERC20Vault();

        nativeTokens = new NativeTokens(vault, outboundChannel, origin);
        vault.transferOwnership(address(nativeTokens));

        account1 = new SovereignAccountMock();
        account2 = address(this);

        token.mint(address(account1), 500);
        token.mint(address(account2), 500);
    }

    function testHandleRevertsUnknownOrigin() public {
        NativeTokens.Message memory message;
        bytes32 unknownOrigin = keccak256("UNKNOWN_ORIGIN");
        vm.expectRevert(NativeTokens.UnauthorizedOrigin.selector);
        nativeTokens.handle(unknownOrigin, abi.encode(message));
    }

    function testHandleRevertsUnknownSender() public {
        nativeTokens.transferOwnership(address(account1));
        NativeTokens.Message memory message;
        vm.expectRevert("Ownable: caller is not the owner");
        nativeTokens.handle(origin, abi.encode(message));
    }

    function testHandleUnlockMessageSuccessful() public {
        testLockSuccessful();

        vm.expectEmit(false, false, false, true, address(nativeTokens));
        emit Unlocked(origin, address(account1), address(token), 50);

        vm.expectEmit(false, false, false, true, address(vault));
        emit Withdraw(address(nativeTokens), address(account1), address(token), 50);

        NativeTokens.UnlockPayload memory payload;
        payload.token = address(token);
        payload.recipient = address(account1);
        payload.amount = 50;

        NativeTokens.Message memory message;
        message.action = NativeTokens.Action.Unlock;
        message.payload = abi.encode(payload);
        nativeTokens.handle(origin, abi.encode(message));

        assertEq(token.balanceOf(address(account1)), 550);
        assertEq(token.balanceOf(address(account2)), 450);
        assertEq(token.balanceOf(address(vault)), 0);
        assertEq(token.allowance(address(account2), address(vault)), 50);
        assertEq(vault.balances(address(token)), 0);
    }

    function testLockRevertsZeroAmount() public {
        vm.expectRevert(NativeTokens.ZeroAmount.selector);
        nativeTokens.lock(address(0), bytes32(0), 0);
    }

    function testLockSuccessful() public {
        token.approve(address(vault), 100);

        vm.expectEmit(false, false, false, true, address(vault));
        emit Deposit(address(nativeTokens), address(account2), address(token), 50);

        vm.expectEmit(false, false, false, true, address(nativeTokens));
        emit Locked(address(account2), recipient, address(token), 50);

        bytes memory call;
        vm.expectEmit(false, false, false, true, address(outboundChannel));
        emit Message(address(account2), call, 1_000_000);

        nativeTokens.lock(address(token), recipient, 50);

        assertEq(token.balanceOf(address(account2)), 450);
        assertEq(token.balanceOf(address(vault)), 50);
        assertEq(token.allowance(address(account2), address(vault)), 50);
        assertEq(vault.balances(address(token)), 50);
    }

    function testCreateSuccessful() public {
        bytes memory call;
        vm.expectEmit(false, false, false, true, address(nativeTokens));
        emit Created(address(token), "Test", "T", 18);

        vm.expectEmit(false, false, false, true, address(outboundChannel));
        emit Message(address(this), call, 1_000_000);

        nativeTokens.create(address(token));
    }
}
