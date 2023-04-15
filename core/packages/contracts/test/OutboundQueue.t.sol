// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {Test} from "forge-std/Test.sol";

import {OutboundQueue} from "../src/OutboundQueue.sol";
import {OutboundQueueDelegate} from "../src/OutboundQueueDelegate.sol";
import {Vault} from "../src/Vault.sol";
import {ParaID} from "../src/Types.sol";

contract OutboundQueueTest is Test {
    Vault public vault;
    OutboundQueue public queue;
    OutboundQueueDelegate public delegate;

    ParaID dest = ParaID.wrap(1001);
    bytes messagePayload = bytes("payload");

    function setUp() public {
        vault = new Vault();
        queue = new OutboundQueue();
        delegate = new OutboundQueueDelegate(address(queue), vault, 1 ether);
        queue.grantRole(queue.SUBMIT_ROLE(), address(this));
        queue.updateDelegate(delegate);
    }

    function testSubmit() public {
        queue.submit{value: 1 ether}(dest, hex"", messagePayload);
        vault.balances(dest);
        assertEq(vault.balances(dest), 1 ether);
    }

    function testSubmitFailFeePaymentTooLow() public {
        vm.expectRevert(OutboundQueueDelegate.FeePaymentToLow.selector);
        queue.submit{value: 0.5 ether}(dest, hex"", messagePayload);
        assertEq(vault.balances(dest), 0 ether);
    }
}
