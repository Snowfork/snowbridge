// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

library Decoder {
    // Decodes a SCALE encoded uint256 by converting bytes (bid endian) to little endian format
    function decodeUint256(bytes memory data) public pure returns (uint256) {
        uint256 number;
        for (uint256 i = data.length; i > 0; i--) {
            number = number + uint256(uint8(data[i - 1])) * (2**(8 * (i - 1)));
        }
        return number;
    }

    // Slice a section from a byte array.
    // Inspired by github.com/GNSPS/solidity-bytes-utils/blob/master/contracts/BytesLib.sol.
    function slice(
        bytes memory _bytes,
        uint256 _start,
        uint256 _length
    ) internal pure returns (bytes memory) {
        require(_bytes.length >= (_start + _length), "Read out of bounds");

        bytes memory temp;

        assembly {
            switch iszero(_length)
                case 0 {
                    // Get a location of some free memory and store it in temp
                    temp := mload(0x40)

                    let lengthmod := and(_length, 31)

                    // Multiply to prevent the copy loop from ending prematurely
                    let mc := add(
                        add(temp, lengthmod),
                        mul(0x20, iszero(lengthmod))
                    )
                    let end := add(mc, _length)

                    for {
                        // Again, multiply to prevent the copy loop from ending prematurely
                        let cc := add(
                            add(
                                add(_bytes, lengthmod),
                                mul(0x20, iszero(lengthmod))
                            ),
                            _start
                        )
                    } lt(mc, end) {
                        mc := add(mc, 0x20)
                        cc := add(cc, 0x20)
                    } {
                        mstore(mc, mload(cc))
                    }

                    mstore(temp, _length)
                    // Allocate the array padded to 32 bytes
                    mstore(0x40, and(add(mc, 31), not(31)))
                }
                // If we want a zero-length slice let's just return a zero-length array
                default {
                    temp := mload(0x40)
                    mstore(0x40, add(temp, 0x20))
                }
        }
        return temp;
    }

    // Slice 20 bytes and cast to an address
    function sliceAddress(bytes memory _bytes, uint256 _start)
        internal
        pure
        returns (address payable)
    {
        require(_bytes.length >= (_start + 20), "Read out of bounds");

        address payable temp;
        assembly {
            temp := div(
                mload(add(add(_bytes, 0x20), _start)),
                0x1000000000000000000000000
            )
        }
        return temp;
    }
}
