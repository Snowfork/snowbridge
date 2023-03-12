// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import { OutboundChannel } from "../OutboundChannel.sol";
import { Vault } from "../Vault.sol";

contract OutboundChannelTest is Test {
    Vault public vault;
    OutboundChannel public channel;

    bytes dest = bytes("statemint");
    bytes message = bytes("message");

    function setUp() public {
        vault = new Vault();
        channel = new OutboundChannel(vault, 1 ether);
        channel.grantRole(channel.SUBMIT_ROLE(), address(this));
    }

    function testSubmit() public {
        channel.submit{ value: 1 ether }(dest, message);
        vault.balances(dest);
        assertEq(vault.balances(dest), 1 ether);
    }

    function testSubmitFailFeePaymentTooLow() public {
        vm.expectRevert(OutboundChannel.FeePaymentToLow.selector);
        channel.submit{ value: 0.5 ether }(dest, message);
        assertEq(vault.balances(dest), 0 ether);
    }
}
