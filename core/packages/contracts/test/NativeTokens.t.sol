// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/console2.sol";

import "canonical-weth/WETH9.sol";

import "../src/NativeTokens.sol";
import "../src/TokenVault.sol";

import "./mocks/OutboundQueueMock.sol";
import "./mocks/TestToken.sol";

contract NativeTokensTest is Test {
    event Locked(bytes recipient, address token, uint128 amount);
    event Unlocked(address recipient, address token, uint128 amount);
    event Created(address token);

    TokenVault private vault;
    NativeTokens private nativeTokens;
    IOutboundQueue private outboundQueue;
    WETH9 private token;
    address private account1;
    address private account2;

    bytes private constant peer = "/Polkadot/Para(Statemint)";
    bytes private constant recipient = "/Alice";

    function setUp() public {
        token = new WETH9();

        outboundQueue = new OutboundQueueMock();
        vault = new TokenVault();
        nativeTokens = new NativeTokens(vault, outboundQueue, peer, 1);
        vault.grantRole(vault.WITHDRAW_ROLE(), address(nativeTokens));
        vault.grantRole(vault.DEPOSIT_ROLE(), address(nativeTokens));

        account1 = makeAddr("account1");
        account2 = address(this);

        nativeTokens.grantRole(nativeTokens.SENDER_ROLE(), address(this));

        // create tokens for account 1
        hoax(account1);
        token.deposit{value: 500}();

        // create tokens for account 2
        token.deposit{value: 500}();
    }

    function testHandleRevertsUnknownOrigin() public {
        NativeTokens.Message memory message;
        bytes memory unknownOrigin = "UNKNOWN_ORIGIN";
        vm.expectRevert(NativeTokens.Unauthorized.selector);
        nativeTokens.handle(unknownOrigin, abi.encode(message));
    }

    function testHandleRevertsUnknownSender() public {
        NativeTokens.Message memory message;
        nativeTokens.revokeRole(nativeTokens.SENDER_ROLE(), address(this));
        vm.expectRevert();
        nativeTokens.handle(peer, abi.encode(message));
    }

    function testHandleUnlockMessageSuccessful() public {
        testLockSuccessful();

        vm.expectEmit(false, false, false, true, address(nativeTokens));
        emit Unlocked(address(account1), address(token), 50);

        NativeTokens.UnlockPayload memory payload;
        payload.token = address(token);
        payload.recipient = address(account1);
        payload.amount = 50;

        NativeTokens.Message memory message;
        message.action = NativeTokens.Action.Unlock;
        message.payload = abi.encode(payload);
        nativeTokens.handle(peer, abi.encode(message));

        assertEq(token.balanceOf(address(account1)), 550);
        assertEq(token.balanceOf(address(account2)), 450);
        assertEq(token.balanceOf(address(vault)), 0);
        assertEq(token.allowance(address(account2), address(vault)), 50);
        assertEq(vault.balance(address(token)), 0);
    }

    function testLockRevertsZeroAmount() public {
        vm.expectRevert(NativeTokens.InvalidAmount.selector);
        nativeTokens.lock(address(token), recipient, 0);
    }

    function testLockSuccessful() public {
        token.approve(address(vault), 100);

        vm.expectEmit(false, false, false, true, address(nativeTokens));
        emit Locked(recipient, address(token), 50);

        nativeTokens.lock(address(token), recipient, 50);

        assertEq(token.balanceOf(address(account2)), 450);
        assertEq(token.balanceOf(address(vault)), 50);
        assertEq(token.allowance(address(account2), address(vault)), 50);
        assertEq(vault.balance(address(token)), 50);
    }

    function testCreateSuccessful() public {
        uint256 fee = nativeTokens.createTokenFee();

        vm.expectEmit(false, false, false, true, address(nativeTokens));
        emit Created(address(token));

        nativeTokens.create{value: fee}(address(token));
    }

    function testCreateFailOnBadFeePayment() public {
        uint256 fee = nativeTokens.createTokenFee();
        vm.expectRevert(NativeTokens.NoFundsforCreateToken.selector);
        nativeTokens.create{value: fee - 1}(address(this));
    }

    function testCreateFailOnBadToken() public {
        uint256 fee = nativeTokens.createTokenFee();
        vm.expectRevert();
        nativeTokens.create{value: fee}(address(this));
    }
}
