// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {ScaleCodec} from "../src/ScaleCodec.sol";

contract ScaleCodecTest is Test {
    function testEncodeU256() public {
        assertEq(
            ScaleCodec.encodeU256(12063978950259949786323707366460749298097791896371638493358994162204017315152),
            hex"504d8a21dd3868465c8c9f2898b7f014036935fa9a1488629b109d3d59f8ab1a"
        );
    }

    function testEncodeU128() public {
        assertEq(ScaleCodec.encodeU128(35452847761173902980759433963665451267), hex"036935fa9a1488629b109d3d59f8ab1a");
    }

    function testEncodeU64() public {
        assertEq(ScaleCodec.encodeU64(1921902728173129883), hex"9b109d3d59f8ab1a");
    }

    function testEncodeU32() public {
        assertEq(ScaleCodec.encodeU32(447477849), hex"59f8ab1a");
    }

    function testEncodeU16() public {
        assertEq(ScaleCodec.encodeU16(6827), hex"ab1a");
    }
}
