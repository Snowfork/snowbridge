// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {Counter} from "../src/utils/Counter.sol";

contract CounterTest is Test {
    using Counter for uint256[];

    uint256[] counters;

    function setUp() public {
        delete counters;
    }

    function testCounterCreatedInitializationRoundsUp() public {
        // 33 uint16s will require 3 uint256s
        uint256[] memory expected = new uint256[](3);
        counters = Counter.createCounter(33);
        assertEq(counters, expected);
    }

    function testCounterCreatedAsZeroed() public {
        uint256[] memory expected = new uint256[](2);
        counters = Counter.createCounter(32);
        counters[0] = 0xABABABAB;
        counters[1] = 0xABABABAB;
        counters = Counter.createCounter(32);
        assertEq(counters, expected);
    }

    function testCounterSet() public {
        uint256[] memory expected = new uint256[](2);

        // Manually set the 16th index to 2.
        expected[1] = 2;

        counters = Counter.createCounter(32);
        counters.set(16, 2);

        assertEq(counters, expected);
    }

    function testCounterGet() public {
        counters = Counter.createCounter(32);

        // Manually set the 16th index to 2.
        counters[1] = 2;

        assertEq(counters.get(16), 2);
    }

    function testCounterGetAndSetAlongEntireRange() public {
        counters = Counter.createCounter(32);
        for (uint16 index = 0; index < 32; index++) {
            // Should be zero as the initial value.
            uint16 value = counters.get(index);
            assertEq(value, 0);

            counters.set(index, index);
            value = counters.get(index);
            assertEq(value, index);
        }
    }
}
