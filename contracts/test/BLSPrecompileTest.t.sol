// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

/**
 * @title BLSPrecompileTest
 * @dev Test to check if BLS12-381 precompiles (EIP-2537) are available
 */
contract BLSPrecompileTest is Test {
    // BLS12-381 precompile addresses
    address constant G1_ADD = address(0x0a);
    address constant G1_MUL = address(0x0b);
    address constant G1_MULTIEXP = address(0x0c);
    address constant G2_ADD = address(0x0d);
    address constant G2_MUL = address(0x0e);
    address constant G2_MULTIEXP = address(0x0f);
    address constant PAIRING = address(0x10);
    address constant MAP_FP_TO_G1 = address(0x11);
    address constant MAP_FP2_TO_G2 = address(0x12);

    function testBLSPrecompilesAvailable() public {
        console.log("Checking BLS12-381 precompile availability...");

        // Check if precompiles have code (they should if implemented)
        uint256 g1AddSize;
        uint256 pairingSize;

        assembly {
            g1AddSize := extcodesize(0x0a)
            pairingSize := extcodesize(0x10)
        }

        console.log("G1_ADD (0x0a) code size:", g1AddSize);
        console.log("PAIRING (0x10) code size:", pairingSize);

        if (g1AddSize > 0 || pairingSize > 0) {
            console.log("BLS precompiles appear to be available!");
        } else {
            console.log("BLS precompiles NOT available (code size = 0)");
        }

        // Try a simple call to see if it reverts
        bytes memory input = hex"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        (bool success, bytes memory result) = G1_ADD.staticcall(input);

        console.log("G1_ADD call success:", success);
        console.log("G1_ADD result length:", result.length);

        if (!success) {
            console.log("BLS precompiles are NOT available in this environment");
            console.log("EIP-2537 has not been activated");
        } else {
            console.log("BLS precompiles ARE available!");
        }
    }
}
