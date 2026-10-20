// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {Uint16Array} from "../src/utils/Uint16Array.sol";

contract Uint16ArrayTest is Test {
    Uint16Array counters;

    function testCounterSet() public {
        counters.set(16, 2);

        // The 16th index is the lowest counter of the second word.
        assertEq(counters.data[0], 0);
        assertEq(counters.data[1], 2);
    }

    function testCounterGet() public {
        // Manually set the 16th index to 2.
        counters.data[1] = 2;

        assertEq(counters.get(16), 2);
    }

    function testCounterGetAndSetAlongEntireRange() public {
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

    function testCounterWordsNeedNoAllocation() public {
        // Any index is usable without creating the array first; unused words stay zero.
        counters.set(599, 7);
        assertEq(counters.get(599), 7);
        assertEq(counters.get(598), 0);
        // Index 599 is counter 7 of word 37.
        assertEq(counters.data[37], uint256(7) << (16 * 7));
    }
}
