// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {BitfieldWrapper} from "./mocks/BitfieldWrapper.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

import {stdJson} from "forge-std/StdJson.sol";

contract BitfieldTest is Test {
    using stdJson for string;

    function testBitfieldSubsampling() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        string memory json =
            vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json"));
        uint32 setSize = uint32(json.readUint(".validatorSetSize"));
        uint256[] memory bitSetArray = json.readUintArray(".participants");

        uint256[] memory initialBitField = bw.createBitfield(bitSetArray, setSize);
        uint256[] memory finalBitfield = bw.subsample(67, initialBitField, 30, setSize);

        uint256 counter = 0;
        for (uint256 i = 0; i < bitSetArray.length; i++) {
            if (Bitfield.isSet(finalBitfield, bitSetArray[i])) {
                counter++;
            }
        }
        assertEq(30, counter);
        assertEq(Bitfield.countSetBits(finalBitfield), counter);
    }
    
    function testBitfieldWithZeroLength() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        
        // Empty bitfield with zero length
        uint256[] memory emptyBits;
        emptyBits = new uint256[](0);
        
        // This should create a valid bitfield with 0 length
        uint256[] memory initialBitField = bw.createBitfield(emptyBits, 0);
        
        // When length is 0, subsample should handle it gracefully without infinite loop
        // Since we're asking for 0 bits, it should return an empty bitfield
        uint256[] memory finalBitfield = bw.subsample(67, initialBitField, 0, 0);
        
        // Ensure the returned bitfield has the expected length and no set bits
        assertEq(finalBitfield.length, initialBitField.length);
        assertEq(Bitfield.countSetBits(finalBitfield), 0);
    }
}
