// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

contract Scale {

    function readByteAtIndex(bytes memory data, uint8 index)
        public
        pure
        returns (uint8)
    {
        return uint8(data[index]);
    }

    // Converts uint8 into bytes memory with length 4
    function uint8ToBytes(uint8 x)
        public
        pure
        returns (bytes memory b)
    {
        b = new bytes(4);
        assembly { mstore(add(b, 4), x) }
    }

    // Converts uint64 input into bytes[4+input]
    function uint64ToBytes(uint64 x)
        public
        pure
        returns (bytes memory b)
    {
        uint256 len = x + 4;
        b = new bytes(len);
        assembly { mstore(add(b, len), x) }
    }

    function toByte4(bytes memory _b)
        public
        pure
        returns (bytes4 _result)
    {
        assembly {
            _result := mload(add(_b, 0x4))
        }
    }

    function toBytesMemory(bytes1 _b)
        public
        pure
        returns (bytes memory _result)
    {
        assembly {
            _result := mload(add(_b, 0x1))
        }
    }

    // Inspired by https://golang.org/src/encoding/binary/binary.go
    function littleEndianUint32(uint32 b32) //(bytes memory b)
        public
        pure
        returns (uint32)
    {
        // TODO: this attempt to reverse bytes to build little endian isn't gonna work.
        // bytes memory leBytes = reverse(toBytesMemory(beBytes));
        // uint32 b32 = uint32(toByte4(leBytes));

        // Build little endian
        // bytes memory output = new bytes(4);
        // output[0] = byte(b32);
        // output[1] = byte(b32 >> 8);
        // output[2] = byte(b32 >> 16);
        // output[3] = byte(b32 >> 24);

        uint32 second = b32 >> 8;
        uint32 third = b32 >> 16;
        uint32 forth = b32 >> 24;

        uint32 firstXOR = b32 | second << 8;
        uint32 secondXOR = firstXOR | third << 16;
        uint32 thirdXOR = secondXOR | forth << 24;
	    return thirdXOR;
    }

    function reverse(bytes memory input)
        public
        pure
        returns(bytes memory)
    {
        bytes memory reversed = new bytes(input.length);
        for(uint i = 0; i < input.length; i++) {
            reversed[input.length - i - 1] = input[i];
        }
        return reversed;
    }

    function bytesToUint256(bytes memory b)
        public
        pure
        returns (uint256)
    {
        uint256 number;
        for(uint i=0;i<b.length;i++){
            number = number +  uint(uint8(b[i]))*(2**(8*(b.length-(i+1))));
        }
        return number;
    }

    // Decodes a SCALE encoded compact unsigned integer
    function decodeUintCompact(bytes memory data)
        public
        pure
        returns (uint256)
    {
        uint8 b = readByteAtIndex(data, 0);         // read the first byte
        uint8 mode = b & 3;                         // bitwise operation

        if(mode == 0) {
            return b >> 2;                          // right shift to remove mode bits
        } else if(mode == 1) {
            uint8 bb = readByteAtIndex(data, 1);    // read the second byte
            uint64 r = bb;                          // convert to uint64
            r <<= 6;                                // multiply by * 2^6
            r += b >> 2;                            // right shift to remove mode bits
            return r;
        } else if(mode == 2) {
		    uint32 r = littleEndianUint32(b); // set the buffer in little endian order
    		r >>= 2;                                // remove the last 2 mode bits
    		return r;
        } else if(mode == 3) {
            uint64 l = b >> 2;                      // TODO: uint64 or uint256?
            require(l <= 63,                        // Max upper bound of 536 is (67 - 4)
                    "Not supported: l>63 encountered when decoding a compact-encoded uint");
			bytes memory buf = uint64ToBytes(l);    // convert to bytes
			bytes memory reverseBuf = reverse(buf); // reverse byte array
			return bytesToUint256(reverseBuf);      // convert reversed bytes to uint256

        } else {
            revert("Code should be unreachable");
        }
    }

}
