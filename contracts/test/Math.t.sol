// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Test} from "forge-std/Test.sol";

import {Math} from "../src/utils/Math.sol";

contract MathTest is Test {
    struct Log2Test {
        uint256 result;
        uint256 input;
    }

    function setUp() public {}

    function testLog2WithWellKnownValues() public pure {
        // Test log will well known values generated from python.
        Log2Test[47] memory tests = [
            Log2Test({result: 0, input: 0}),
            Log2Test({result: 0, input: 1}),
            Log2Test({result: 1, input: 2}),
            Log2Test({result: 2, input: 3}),
            Log2Test({result: 2, input: 4}),
            Log2Test({result: 3, input: 5}),
            Log2Test({result: 3, input: 6}),
            Log2Test({result: 3, input: 8}),
            Log2Test({result: 4, input: 9}),
            Log2Test({result: 4, input: 12}),
            Log2Test({result: 4, input: 16}),
            Log2Test({result: 5, input: 17}),
            Log2Test({result: 5, input: 24}),
            Log2Test({result: 5, input: 32}),
            Log2Test({result: 6, input: 33}),
            Log2Test({result: 6, input: 48}),
            Log2Test({result: 6, input: 64}),
            Log2Test({result: 7, input: 65}),
            Log2Test({result: 7, input: 96}),
            Log2Test({result: 7, input: 128}),
            Log2Test({result: 8, input: 129}),
            Log2Test({result: 8, input: 192}),
            Log2Test({result: 8, input: 256}),
            Log2Test({result: 9, input: 257}),
            Log2Test({result: 9, input: 384}),
            Log2Test({result: 9, input: 512}),
            Log2Test({result: 10, input: 513}),
            Log2Test({result: 10, input: 768}),
            Log2Test({result: 10, input: 1024}),
            Log2Test({result: 11, input: 1025}),
            Log2Test({result: 11, input: 1536}),
            Log2Test({result: 11, input: 2048}),
            Log2Test({result: 12, input: 2049}),
            Log2Test({result: 12, input: 3072}),
            Log2Test({result: 12, input: 4096}),
            Log2Test({result: 13, input: 4097}),
            Log2Test({result: 13, input: 6144}),
            Log2Test({result: 13, input: 8192}),
            Log2Test({result: 14, input: 8193}),
            Log2Test({result: 14, input: 12_288}),
            Log2Test({result: 14, input: 16_384}),
            Log2Test({result: 15, input: 16_385}),
            Log2Test({result: 15, input: 24_576}),
            Log2Test({result: 15, input: 32_768}),
            Log2Test({result: 16, input: 32_769}),
            Log2Test({result: 16, input: 49_152}),
            Log2Test({result: 16, input: 65_535})
        ];

        for (uint256 t = 0; t < tests.length; ++t) {
            assertEq(tests[t].result, Math.log2(tests[t].input, Math.Rounding.Ceil));
        }
    }

    function testFuzzMin(uint256 a, uint256 b) public pure {
        vm.assume(a < b);
        assertEq(a, Math.min(a, b));
    }

    function testFuzzMax(uint256 a, uint256 b) public pure {
        vm.assume(a > b);
        assertEq(a, Math.max(a, b));
    }

    function testFuzzSaturatingAdd(uint16 a, uint16 b) public pure {
        uint256 result = uint256(a) + uint256(b);
        if (result > 0xFFFF) {
            result = 0xFFFF;
        }
        assertEq(result, Math.saturatingAdd(a, b));
    }

    function testFuzzSaturatingSub(uint256 a, uint256 b) public pure {
        uint256 result = 0;
        if (a > b) {
            result = a - b;
        }
        assertEq(result, Math.saturatingSub(a, b));
    }
}
