// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {Test} from "forge-std/Test.sol";

import {InboundQueue} from "../src/InboundQueue.sol";
import {InboundQueueDelegate} from "../src/InboundQueueDelegate.sol";
import {Vault} from "../src/Vault.sol";
import {IParachainClient} from "../src/IParachainClient.sol";
import {ParaID} from "../src/Types.sol";
import {ParachainClientMock} from "./mocks/ParachainClientMock.sol";
import {IRecipient, RecipientMock} from "./mocks/RecipientMock.sol";

contract InboundQueueTest is Test {
    InboundQueue public queue;
    InboundQueueDelegate public delegate;
    RecipientMock public recipient;

    Vault public vault;

    event MessageReceived(ParaID indexed origin, uint64 indexed nonce, bool success);

    ParaID public constant ORIGIN = ParaID.wrap(1001);

    bytes public messagePayload = bytes("message");
    bytes32[] public messageProof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes public parachainHeaderProof = bytes("validProof");

    function setUp() public {
        IParachainClient parachainClient = new ParachainClientMock();
        recipient = new RecipientMock();

        vault = new Vault();

        deal(address(this), 100 ether);

        queue = new InboundQueue();
        delegate = new InboundQueueDelegate(address(queue), parachainClient, vault, 1 ether);
        delegate.updateHandler(1, IRecipient(recipient));
        queue.updateDelegate(delegate);
        vault.grantRole(vault.WITHDRAW_ROLE(), address(delegate));
    }

    function testSubmit() public {
        vault.deposit{value: 50 ether}(ORIGIN);

        bytes memory submitParams = abi.encode(
            InboundQueueDelegate.Message(ORIGIN, 1, 1, messagePayload),
            messageProof,
            parachainHeaderProof
        );

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        queue.submit(submitParams);

        assertEq(vault.balances(ORIGIN), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }

    function testSubmitShouldFailInvalidProof() public {
        vault.deposit{value: 50 ether}(ORIGIN);

        bytes memory submitParams = abi.encode(
            InboundQueueDelegate.Message(ORIGIN, 1, 1, messagePayload),
            messageProof,
            bytes("badProof")
        );

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        vm.expectRevert(InboundQueueDelegate.InvalidProof.selector);
        queue.submit(submitParams);
    }

    function testSubmitShouldFailInvalidNonce() public {
        vault.deposit{value: 50 ether}(ORIGIN);

        bytes memory submitParams = abi.encode(
            InboundQueueDelegate.Message(ORIGIN, 2, 1, messagePayload),
            messageProof,
            parachainHeaderProof
        );

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        vm.expectRevert(InboundQueueDelegate.InvalidNonce.selector);
        queue.submit(submitParams);
    }

    // Test that submission fails if origin does not have sufficient funds to pay relayer
    function testSubmitShouldFailInsufficientBalance() public {
        vault.deposit{value: 0.1 ether}(ORIGIN);

        bytes memory submitParams = abi.encode(
            InboundQueueDelegate.Message(ORIGIN, 1, 1, messagePayload),
            messageProof,
            parachainHeaderProof
        );

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        vm.expectRevert(Vault.InsufficientBalance.selector);
        queue.submit(submitParams);
    }

    function testSubmitShouldNotFailOnHandlerFailure() public {
        vault.deposit{value: 50 ether}(ORIGIN);

        bytes memory submitParams = abi.encode(
            InboundQueueDelegate.Message(ORIGIN, 1, 1, messagePayload),
            messageProof,
            parachainHeaderProof
        );

        recipient.setShouldFail();
        vm.expectEmit(true, true, false, true);
        emit MessageReceived(ORIGIN, 1, false);

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        queue.submit(submitParams);

        assertEq(vault.balances(ORIGIN), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }

    function testSubmitShouldNotFailOnHandlerOOG() public {
        vault.deposit{value: 50 ether}(ORIGIN);

        bytes memory submitParams = abi.encode(
            InboundQueueDelegate.Message(ORIGIN, 1, 1, messagePayload),
            messageProof,
            parachainHeaderProof
        );

        recipient.setShouldConsumeAllGas();
        vm.expectEmit(true, true, false, true);
        emit MessageReceived(ORIGIN, 1, false);

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        queue.submit(submitParams);

        assertEq(vault.balances(ORIGIN), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }
}
