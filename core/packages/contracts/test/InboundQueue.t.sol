// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {InboundQueue} from "../src/InboundQueue.sol";
import {Vault} from "../src/Vault.sol";
import {IParachainClient} from "../src/IParachainClient.sol";
import {ParachainClientMock} from "./mocks/ParachainClientMock.sol";
import {RecipientMock} from "./mocks/RecipientMock.sol";

contract InboundQueueTest is Test {
    InboundQueue public channel;
    RecipientMock public recipient;

    Vault public vault;

    event MessageDispatched(bytes origin, uint64 nonce, InboundQueue.DispatchResult result);

    bytes origin = bytes("statemint");
    bytes32[] proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes message = bytes("message");
    bytes parachainHeaderProof = bytes("validProof");

    function setUp() public {
        IParachainClient parachainClient = new ParachainClientMock();
        recipient = new RecipientMock();

        vault = new Vault();

        deal(address(this), 100 ether);

        channel = new InboundQueue(parachainClient, vault, 1 ether);
        channel.updateHandler(1, InboundQueue.Handler(address(recipient), 60000));
        vault.grantRole(vault.WITHDRAW_ROLE(), address(channel));
    }

    function testSubmit() public {
        vault.deposit{value: 50 ether}(origin);

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        channel.submit(InboundQueue.Message(origin, 1, 1, message), proof, parachainHeaderProof);

        assertEq(vault.balances(origin), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }

    function testSubmitShouldFailInvalidProof() public {
        vault.deposit{value: 50 ether}(origin);

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        vm.expectRevert(InboundQueue.InvalidProof.selector);
        channel.submit(InboundQueue.Message(origin, 1, 1, message), proof, bytes("badProof"));
    }

    function testSubmitShouldFailInvalidNonce() public {
        vault.deposit{value: 50 ether}(origin);

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        vm.expectRevert(InboundQueue.InvalidNonce.selector);
        channel.submit(InboundQueue.Message(origin, 2, 1, message), proof, parachainHeaderProof);
    }

    // Test that submission fails if origin does not have sufficient funds to pay relayer
    function testSubmitShouldFailInsufficientBalance() public {
        vault.deposit{value: 0.1 ether}(origin);

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        vm.expectRevert(Vault.InsufficientBalance.selector);
        channel.submit(InboundQueue.Message(origin, 1, 1, message), proof, parachainHeaderProof);
    }

    function testSubmitShouldNotFailOnHandlerFailure() public {
        vault.deposit{value: 50 ether}(origin);

        recipient.setShouldFail();
        vm.expectEmit(false, false, false, true);
        emit MessageDispatched(origin, 1, InboundQueue.DispatchResult(false, "failed", 0, hex""));

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        channel.submit(InboundQueue.Message(origin, 1, 1, message), proof, parachainHeaderProof);

        assertEq(vault.balances(origin), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }

    function testSubmitShouldNotFailOnHandlerPanic() public {
        vault.deposit{value: 50 ether}(origin);

        recipient.setShouldPanic();
        vm.expectEmit(false, false, false, true);
        emit MessageDispatched(origin, 1, InboundQueue.DispatchResult(false, "", 1, hex""));

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        channel.submit(InboundQueue.Message(origin, 1, 1, message), proof, parachainHeaderProof);

        assertEq(vault.balances(origin), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }

    function testSubmitShouldNotFailOnHandlerOOG() public {
        vault.deposit{value: 50 ether}(origin);

        recipient.setShouldConsumeAllGas();
        vm.expectEmit(false, false, false, true);
        emit MessageDispatched(origin, 1, InboundQueue.DispatchResult(false, "", 0, hex""));

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        channel.submit(InboundQueue.Message(origin, 1, 1, message), proof, parachainHeaderProof);

        assertEq(vault.balances(origin), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }
}
