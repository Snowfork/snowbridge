// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {SparseBitmap} from "../src/utils/SparseBitmap.sol";

contract SparseBitmapTest is Test {
    SparseBitmap bitmap;

    function setUp() public {}

    function testGetSet() public {
        bitmap.set(384);
        assertEq(bitmap.get(384), true);
    }
}
