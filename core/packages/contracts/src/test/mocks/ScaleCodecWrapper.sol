// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../ScaleCodec.sol";

contract ScaleCodecWrapper {
    function encodeU256(uint256 input) external pure returns (bytes32) {
        return ScaleCodec.encodeU256(input);
    }

    function encodeU128(uint128 input) external pure returns (bytes16) {
        return ScaleCodec.encodeU128(input);
    }

    function encodeU64(uint64 input) external pure returns (bytes8) {
        return ScaleCodec.encodeU64(input);
    }

    function encodeU32(uint32 input) external pure returns (bytes4) {
        return ScaleCodec.encodeU32(input);
    }

    function encodeU16(uint16 input) external pure returns (bytes2) {
        return ScaleCodec.encodeU16(input);
    }

    function encodeU8(uint8 input) external pure returns (bytes1) {
        return ScaleCodec.encodeU8(input);
    }
}
