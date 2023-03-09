// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {InboundChannel} from "../InboundChannel.sol";
import {Vault} from "../Vault.sol";
import {IParachainClient} from "../IParachainClient.sol";
import {ParachainClientMock} from "./mocks/ParachainClientMock.sol";
import {RecipientMock} from "./mocks/RecipientMock.sol";

contract InboundChannelTest is Test {
    InboundChannel public channel;
    RecipientMock public recipient;

    Vault public vault;

    bytes origin = bytes("statemint");
    bytes32[] proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bool[] hashSides = [true];

    function setUp() public {
        IParachainClient parachainClient = new ParachainClientMock();
        recipient = new RecipientMock();

        vault = new Vault();

        deal(address(this), 100 ether);

        channel = new InboundChannel(parachainClient, vault, 1 ether);
        channel.updateHandler(1, recipient);
        vault.grantRole(vault.WITHDRAW_ROLE(), address(channel));
    }

    function testSubmit() public {
        vault.deposit{value: 50 ether}(bytes("statemint"));

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        channel.submit(InboundChannel.Message(origin, 1, 1, 20000, hex"deadbeef"), proof, hex"deadbeef");

        assertEq(vault.balances(origin), 49 ether);
        assertEq(relayer.balance, 2 ether);
    }

    // Test that submission fails if origin does not have sufficient funds to pay relayer
    function testSubmitShouldFailInsufficientBalance() public {
        vault.deposit{value: 0.1 ether}(bytes("statemint"));

        address relayer = makeAddr("alice");
        hoax(relayer, 1 ether);

        vm.expectRevert(Vault.InsufficientBalance.selector);
        channel.submit(InboundChannel.Message(origin, 1, 1, 20000, hex"deadbeef"), proof, hex"deadbeef");
    }
}
