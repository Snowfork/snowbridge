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

    function testCounterWithLengthNotMultipleOf16() public {
        // 33 uint16s will require 3 uint256s
        uint256[] memory expected = new uint256[](3);
        expected[2] = 1;

        counters = Counter.createCounter(33);
        counters.set(32, counters.get(32) + 1);
        assertEq(counters, expected);
    }

    function testCounterCreatedAsZeroed() public {
        uint256[] memory expected = new uint256[](2);
        counters = Counter.createCounter(16);
        counters[0] = 0xABABABAB;
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
            assertEq(value, 0, "initially zeroed.");

            if (index > 1) {
                value = counters.get(index - 1);
                assertEq(value, index - 1, "check the counter previously set before update");
            }
            counters.set(index, index);
            value = counters.get(index);
            assertEq(value, index, "check counter set now");
            if (index > 1) {
                value = counters.get(index - 1);
                assertEq(value, index - 1, "check previous counter after the current set");
            }
        }
        for (uint16 index = 0; index < 32; index++) {
            uint16 value = counters.get(index) + 1;
            counters.set(index, value);
            assertEq(value, index + 1, "one added.");

            if (index > 1) {
                value = counters.get(index - 1);
                assertEq(value, index, "check previous counter set after second iteration of set");
            }
        }
    }

    function testCounterGetAndSetWithTwoIterations() public {
        counters = Counter.createCounter(300);
        uint256 index = 0;
        uint16 value = 11;
        counters.set(index, value);
        uint16 new_value = counters.get(index);
        console.log("round1:index at %d set %d and get %d", index, value, new_value);
        assertEq(value, new_value);
        value = value + 1;
        counters.set(index, value);
        new_value = counters.get(index);
        console.log("round2:index at %d set %d and get %d", index, value, new_value);
        assertEq(value, new_value);
    }
}
