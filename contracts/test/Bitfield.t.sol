// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {BitfieldWrapper} from "./mocks/BitfieldWrapper.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

import {stdJson} from "forge-std/StdJson.sol";

contract BitfieldTest is Test {
    using stdJson for string;

    function testBitfieldSubsampling() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        string memory json = vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json"));
        uint32 setSize = uint32(json.readUint(".validatorSetSize"));
        bytes32 root = json.readBytes32(".validatorRoot");
        uint256[] memory bitSetArray = json.readUintArray(".participants");

        uint256[] memory initialBitField = bw.createBitfield(bitSetArray, setSize);
        uint256[] memory finalBitfield = bw.subsample(67, initialBitField, 10, setSize);

        uint256 counter = 0;
        for (uint256 i = 0; i < bitSetArray.length; i++) {
            if (Bitfield.isSet(finalBitfield, bitSetArray[i])) {
                counter++;
            }
        }
        assertEq(10, counter);
        assertEq(Bitfield.countSetBits(finalBitfield), counter);
    }
}
