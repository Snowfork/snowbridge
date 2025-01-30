// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {ScaleCodec} from "../src/utils/ScaleCodec.sol";

contract ScaleCodecTest is Test {
    function testEncodeU256() public {
        assertEq(
            ScaleCodec.encodeU256(
                12_063_978_950_259_949_786_323_707_366_460_749_298_097_791_896_371_638_493_358_994_162_204_017_315_152
            ),
            hex"504d8a21dd3868465c8c9f2898b7f014036935fa9a1488629b109d3d59f8ab1a"
        );
    }

    function testEncodeU128() public {
        assertEq(
            ScaleCodec.encodeU128(35_452_847_761_173_902_980_759_433_963_665_451_267),
            hex"036935fa9a1488629b109d3d59f8ab1a"
        );
    }

    function testEncodeU64() public {
        assertEq(ScaleCodec.encodeU64(1_921_902_728_173_129_883), hex"9b109d3d59f8ab1a");
    }

    function testEncodeU32() public {
        assertEq(ScaleCodec.encodeU32(447_477_849), hex"59f8ab1a");
    }

    function testEncodeU16() public {
        assertEq(ScaleCodec.encodeU16(6827), hex"ab1a");
    }

    function testEncodeCompactU32() public {
        assertEq(ScaleCodec.encodeCompactU32(0), hex"00");
        assertEq(ScaleCodec.encodeCompactU32(63), hex"fc");
        assertEq(ScaleCodec.encodeCompactU32(64), hex"0101");
        assertEq(ScaleCodec.encodeCompactU32(16_383), hex"fdff");
        assertEq(ScaleCodec.encodeCompactU32(16_384), hex"02000100");
        assertEq(ScaleCodec.encodeCompactU32(1_073_741_823), hex"feffffff");
        assertEq(ScaleCodec.encodeCompactU32(1_073_741_824), hex"0300000040");
        assertEq(ScaleCodec.encodeCompactU32(type(uint32).max), hex"03ffffffff");
    }

    function testCheckedEncodeCompactU32() public {
        assertEq(ScaleCodec.checkedEncodeCompactU32(type(uint32).max), hex"03ffffffff");

        vm.expectRevert(ScaleCodec.UnsupportedCompactEncoding.selector);
        ScaleCodec.checkedEncodeCompactU32(uint256(type(uint32).max) + 1);
    }
}
