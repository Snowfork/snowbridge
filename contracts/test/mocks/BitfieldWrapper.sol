// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Bitfield} from "../../src/utils/Bitfield.sol";

contract BitfieldWrapper {
    function createBitfield(uint256[] calldata bitsToSet, uint256 length)
        public
        pure
        returns (uint256[] memory bitfield)
    {
        return Bitfield.createBitfield(bitsToSet, length);
    }

    function subsample(uint256 seed, uint256[] memory prior, uint256 n, uint256 length)
        public
        pure
        returns (uint256[] memory bitfield)
    {
        return Bitfield.subsample(seed, prior, n, length);
    }
}
