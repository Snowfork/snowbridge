// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

library ScaleCodec {
    error UnsupportedCompactEncoding();

    uint256 internal constant MAX_COMPACT_ENCODABLE_UINT = 2 ** 30 - 1;

    // Sources:
    //   * https://ethereum.stackexchange.com/questions/15350/how-to-convert-an-bytes-to-address-in-solidity/50528
    //   * https://graphics.stanford.edu/~seander/bithacks.html#ReverseParallel

    function reverse256(uint256 input) internal pure returns (uint256 v) {
        v = input;

        // swap bytes
        v = ((v & 0xFF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00) >> 8)
            | ((v & 0x00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF) << 8);

        // swap 2-byte long pairs
        v = ((v & 0xFFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000) >> 16)
            | ((v & 0x0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF) << 16);

        // swap 4-byte long pairs
        v = ((v & 0xFFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000) >> 32)
            | ((v & 0x00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF) << 32);

        // swap 8-byte long pairs
        v = ((v & 0xFFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF0000000000000000) >> 64)
            | ((v & 0x0000000000000000FFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF) << 64);

        // swap 16-byte long pairs
        v = (v >> 128) | (v << 128);
    }

    function reverse128(uint128 input) internal pure returns (uint128 v) {
        v = input;

        // swap bytes
        v = ((v & 0xFF00FF00FF00FF00FF00FF00FF00FF00) >> 8)
            | ((v & 0x00FF00FF00FF00FF00FF00FF00FF00FF) << 8);

        // swap 2-byte long pairs
        v = ((v & 0xFFFF0000FFFF0000FFFF0000FFFF0000) >> 16)
            | ((v & 0x0000FFFF0000FFFF0000FFFF0000FFFF) << 16);

        // swap 4-byte long pairs
        v = ((v & 0xFFFFFFFF00000000FFFFFFFF00000000) >> 32)
            | ((v & 0x00000000FFFFFFFF00000000FFFFFFFF) << 32);

        // swap 8-byte long pairs
        v = (v >> 64) | (v << 64);
    }

    function reverse64(uint64 input) internal pure returns (uint64 v) {
        v = input;

        // swap bytes
        v = ((v & 0xFF00FF00FF00FF00) >> 8) | ((v & 0x00FF00FF00FF00FF) << 8);

        // swap 2-byte long pairs
        v = ((v & 0xFFFF0000FFFF0000) >> 16) | ((v & 0x0000FFFF0000FFFF) << 16);

        // swap 4-byte long pairs
        v = (v >> 32) | (v << 32);
    }

    function reverse32(uint32 input) internal pure returns (uint32 v) {
        v = input;

        // swap bytes
        v = ((v & 0xFF00FF00) >> 8) | ((v & 0x00FF00FF) << 8);

        // swap 2-byte long pairs
        v = (v >> 16) | (v << 16);
    }

    function reverse16(uint16 input) internal pure returns (uint16 v) {
        v = input;

        // swap bytes
        v = (v >> 8) | (v << 8);
    }

    function encodeU256(uint256 input) internal pure returns (bytes32) {
        return bytes32(reverse256(input));
    }

    function encodeU128(uint128 input) internal pure returns (bytes16) {
        return bytes16(reverse128(input));
    }

    function encodeU64(uint64 input) internal pure returns (bytes8) {
        return bytes8(reverse64(input));
    }

    function encodeU32(uint32 input) internal pure returns (bytes4) {
        return bytes4(reverse32(input));
    }

    function encodeU16(uint16 input) internal pure returns (bytes2) {
        return bytes2(reverse16(input));
    }

    function encodeU8(uint8 input) internal pure returns (bytes1) {
        return bytes1(input);
    }

    // Supports compact encoding of integers in [0, uint32.MAX]
    function encodeCompactU32(uint32 value) internal pure returns (bytes memory) {
        if (value <= 2 ** 6 - 1) {
            // add single byte flag
            return abi.encodePacked(uint8(value << 2));
        } else if (value <= 2 ** 14 - 1) {
            // add two byte flag and create little endian encoding
            return abi.encodePacked(ScaleCodec.reverse16(uint16(((value << 2) + 1))));
        } else if (value <= 2 ** 30 - 1) {
            // add four byte flag and create little endian encoding
            return abi.encodePacked(ScaleCodec.reverse32(uint32((value << 2)) + 2));
        } else {
            return abi.encodePacked(uint8(3), ScaleCodec.reverse32(value));
        }
    }

    function encodeCompactU128(uint128 value) internal pure returns (bytes memory) {
        // 1) up to 2^6 - 1
        if (value <= 63) {
            // single byte = (value << 2)
            // (lowest two bits = 00)
            return abi.encodePacked(uint8(value << 2));
        }

        // 2) up to 2^14 - 1
        if (value <= 0x3FFF) {
            // two bytes = (value << 2) + 0x01
            // (lowest two bits = 01)
            uint16 encoded = uint16(value << 2) | 0x01;
            // We must store it in little-endian
            return abi.encodePacked(reverse16(encoded));
        }

        // 3) up to 2^30 - 1
        if (value <= 0x3FFF_FFFF) {
            // four bytes = (value << 2) + 0x02
            // (lowest two bits = 10)
            uint32 encoded = (uint32(value) << 2) | 0x02;
            return abi.encodePacked(reverse32(encoded));
        }

        // 4) otherwise
        // big integer => prefix + little-endian bytes (no leading zeros)
        // prefix = 0x03 + ((numValueBytes - 4) << 2)
        //   where numValueBytes is how many bytes needed to represent `value`.
        bytes memory littleEndian = _toLittleEndianNoLeadingZeros(value);
        uint8 len = uint8(littleEndian.length); // number of bytes needed

        // Substrate: prefix's lower 2 bits = 0b11,
        // the remaining upper bits = (len - 4).
        // Combined: prefix = 0x03 + ((len - 4) << 2).
        uint8 prefix = ((len - 4) << 2) | 0x03;

        // Concatenate prefix + actual bytes
        return abi.encodePacked(prefix, littleEndian);
    }

    // Convert `value` into a little-endian byte array with no leading zeros.
    // (Leading zeros in LE = trailing zeros in big-endian.)
    function _toLittleEndianNoLeadingZeros(uint128 value) private pure returns (bytes memory) {
        // Even if value=0, that case is handled above in smaller branches,
        // but let's just handle it gracefully anyway:
        if (value == 0) {
            return hex"00";
        }
        // Temporarily build up to 16 bytes in a buffer.
        bytes memory buf = new bytes(16);
        uint128 current = value;
        uint8 i = 0;
        while (current != 0) {
            buf[i] = bytes1(uint8(current & 0xFF));
            current >>= 8;
            unchecked {
                i++;
            }
        }
        // i is now the actual number of bytes used
        // Copy them into a new array of the correct size
        bytes memory out = new bytes(i);
        for (uint8 j = 0; j < i; j++) {
            out[j] = buf[j];
        }
        return out;
    }

    function checkedEncodeCompactU32(uint256 value) internal pure returns (bytes memory) {
        if (value > type(uint32).max) {
            revert UnsupportedCompactEncoding();
        }
        return encodeCompactU32(uint32(value));
    }
}
