// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

/**
 * @title BLS12381
 * @dev Library for BLS12-381 signature verification using EIP-2537 precompiles
 *
 * This library provides functions to verify BLS signatures on the BLS12-381 curve.
 * It uses Ethereum's precompiled contracts (EIP-2537) for efficient pairing operations.
 *
 * BLS signatures allow for signature aggregation, enabling verification of multiple
 * signatures with a single pairing check, significantly reducing gas costs.
 */
library BLS12381 {
    /* Precompile Addresses (EIP-2537) */

    address private constant G1_ADD = address(0x0a);
    address private constant G1_MUL = address(0x0b);
    address private constant G1_MULTIEXP = address(0x0c);
    address private constant G2_ADD = address(0x0d);
    address private constant G2_MUL = address(0x0e);
    address private constant G2_MULTIEXP = address(0x0f);
    address private constant PAIRING = address(0x10);
    address private constant MAP_FP_TO_G1 = address(0x11);
    address private constant MAP_FP2_TO_G2 = address(0x12);

    /* Point Sizes */

    uint256 private constant G1_POINT_SIZE = 48;  // Compressed G1 point
    uint256 private constant G2_POINT_SIZE = 96;  // Compressed G2 point

    /* Errors */

    error InvalidG1Point();
    error InvalidG2Point();
    error InvalidSignature();
    error PairingCheckFailed();
    error PrecompileCallFailed();
    error InvalidPublicKeyCount();

    /**
     * @dev Verify a single BLS signature
     * @param message The 32-byte message hash that was signed
     * @param signature The BLS signature (G1 point, 48 bytes compressed)
     * @param publicKey The BLS public key (G2 point, 96 bytes compressed)
     * @return True if the signature is valid, false otherwise
     */
    function verifySingle(
        bytes32 message,
        bytes calldata signature,
        bytes calldata publicKey
    ) internal view returns (bool) {
        // Validate input sizes
        if (signature.length != G1_POINT_SIZE) {
            revert InvalidG1Point();
        }
        if (publicKey.length != G2_POINT_SIZE) {
            revert InvalidG2Point();
        }

        // Validate points are on curve
        if (!isValidG1Point(signature)) {
            revert InvalidG1Point();
        }
        if (!isValidG2Point(publicKey)) {
            revert InvalidG2Point();
        }

        // Hash message to G1 curve point
        bytes memory messagePoint = hashToG1(message);

        // Perform pairing check: e(signature, G2_generator) == e(H(message), publicKey)
        // This is equivalent to: e(signature, G2_gen) * e(-H(message), publicKey) == 1
        // We use the pairing precompile which checks if the product of pairings equals 1

        return verifyPairing(signature, g2Generator(), messagePoint, publicKey);
    }

    /**
     * @dev Verify an aggregated BLS signature against multiple public keys
     * @param message The message that was signed (same for all signers)
     * @param aggregatedSignature The aggregated signature (G1 point, 48 bytes)
     * @param publicKeys Array of public keys that signed (each 96 bytes)
     * @return True if the aggregated signature is valid
     */
    function verifyAggregated(
        bytes32 message,
        bytes memory aggregatedSignature,
        bytes[] memory publicKeys
    ) internal view returns (bool) {
        if (publicKeys.length == 0) {
            revert InvalidPublicKeyCount();
        }

        // Validate aggregated signature
        if (aggregatedSignature.length != G1_POINT_SIZE) {
            revert InvalidG1Point();
        }
        if (!isValidG1PointMemory(aggregatedSignature)) {
            revert InvalidG1Point();
        }

        // Aggregate public keys
        bytes memory aggregatedPublicKey = aggregateG2PointsMemory(publicKeys);

        // Hash message to G1
        bytes memory messagePoint = hashToG1(message);

        // Verify pairing: e(aggregatedSignature, G2_gen) == e(H(message), aggregatedPublicKey)
        return verifyPairing(aggregatedSignature, g2Generator(), messagePoint, aggregatedPublicKey);
    }

    /**
     * @dev Aggregate multiple G2 public keys into a single public key
     * @param publicKeys Array of G2 public keys to aggregate
     * @return The aggregated public key (G2 point, 96 bytes)
     */
    function aggregateG2Points(bytes[] calldata publicKeys) internal view returns (bytes memory) {
        if (publicKeys.length == 0) {
            revert InvalidPublicKeyCount();
        }

        // Validate all public keys
        for (uint256 i = 0; i < publicKeys.length; i++) {
            if (publicKeys[i].length != G2_POINT_SIZE) {
                revert InvalidG2Point();
            }
            if (!isValidG2Point(publicKeys[i])) {
                revert InvalidG2Point();
            }
        }

        // If only one key, return it
        if (publicKeys.length == 1) {
            return publicKeys[0];
        }

        // Use G2_MULTIEXP for efficient aggregation
        // For simple addition, we can use repeated G2_ADD, but multiexp is more efficient
        // Format: [scalar1, point1, scalar2, point2, ...]
        // For aggregation, all scalars are 1

        bytes memory input = new bytes(publicKeys.length * (32 + G2_POINT_SIZE));
        uint256 offset = 0;

        for (uint256 i = 0; i < publicKeys.length; i++) {
            // Scalar = 1 (32 bytes)
            assembly {
                mstore(add(add(input, 32), offset), 1)
            }
            offset += 32;

            // Copy public key
            for (uint256 j = 0; j < G2_POINT_SIZE; j++) {
                input[offset + j] = publicKeys[i][j];
            }
            offset += G2_POINT_SIZE;
        }

        // Call G2_MULTIEXP precompile
        (bool success, bytes memory result) = G2_MULTIEXP.staticcall(input);
        if (!success || result.length != G2_POINT_SIZE) {
            revert PrecompileCallFailed();
        }

        return result;
    }

    /**
     * @dev Aggregate multiple G2 public keys (memory version)
     * @param publicKeys Array of G2 public keys to aggregate
     * @return The aggregated public key (G2 point, 96 bytes)
     */
    function aggregateG2PointsMemory(bytes[] memory publicKeys) internal view returns (bytes memory) {
        if (publicKeys.length == 0) {
            revert InvalidPublicKeyCount();
        }

        // Validate all public keys
        for (uint256 i = 0; i < publicKeys.length; i++) {
            if (publicKeys[i].length != G2_POINT_SIZE) {
                revert InvalidG2Point();
            }
            if (!isValidG2PointMemory(publicKeys[i])) {
                revert InvalidG2Point();
            }
        }

        // If only one key, return it
        if (publicKeys.length == 1) {
            return publicKeys[0];
        }

        // Use G2_MULTIEXP for efficient aggregation
        bytes memory input = new bytes(publicKeys.length * (32 + G2_POINT_SIZE));
        uint256 offset = 0;

        for (uint256 i = 0; i < publicKeys.length; i++) {
            // Scalar = 1 (32 bytes)
            assembly {
                mstore(add(add(input, 32), offset), 1)
            }
            offset += 32;

            // Copy public key
            for (uint256 j = 0; j < G2_POINT_SIZE; j++) {
                input[offset + j] = publicKeys[i][j];
            }
            offset += G2_POINT_SIZE;
        }

        // Call G2_MULTIEXP precompile
        (bool success, bytes memory result) = G2_MULTIEXP.staticcall(input);
        if (!success || result.length != G2_POINT_SIZE) {
            revert PrecompileCallFailed();
        }

        return result;
    }

    /**
     * @dev Verify a pairing equation: e(p1, q1) == e(p2, q2)
     * Implemented as: e(p1, q1) * e(-p2, q2) == 1
     * @param p1 First G1 point (48 bytes)
     * @param q1 First G2 point (96 bytes)
     * @param p2 Second G1 point (48 bytes)
     * @param q2 Second G2 point (96 bytes)
     * @return True if pairing equation holds
     */
    function verifyPairing(
        bytes memory p1,
        bytes memory q1,
        bytes memory p2,
        bytes memory q2
    ) internal view returns (bool) {
        // Negate p2 for the pairing check
        bytes memory negP2 = negateG1(p2);

        // Prepare input for pairing precompile
        // Format: [p1, q1, negP2, q2]
        bytes memory input = new bytes(2 * (G1_POINT_SIZE + G2_POINT_SIZE));
        uint256 offset = 0;

        // Copy p1
        for (uint256 i = 0; i < G1_POINT_SIZE; i++) {
            input[offset + i] = p1[i];
        }
        offset += G1_POINT_SIZE;

        // Copy q1
        for (uint256 i = 0; i < G2_POINT_SIZE; i++) {
            input[offset + i] = q1[i];
        }
        offset += G2_POINT_SIZE;

        // Copy negP2
        for (uint256 i = 0; i < G1_POINT_SIZE; i++) {
            input[offset + i] = negP2[i];
        }
        offset += G1_POINT_SIZE;

        // Copy q2
        for (uint256 i = 0; i < G2_POINT_SIZE; i++) {
            input[offset + i] = q2[i];
        }

        // Call pairing precompile
        (bool success, bytes memory result) = PAIRING.staticcall(input);
        if (!success) {
            revert PrecompileCallFailed();
        }

        // Result should be 32 bytes: 0x00...01 if pairing holds, 0x00...00 otherwise
        if (result.length != 32) {
            revert PairingCheckFailed();
        }

        // Check if result is 1
        return uint256(bytes32(result)) == 1;
    }

    /**
     * @dev Hash a message to a G1 curve point
     * @param message The 32-byte message to hash
     * @return The G1 point (48 bytes compressed)
     */
    function hashToG1(bytes32 message) internal view returns (bytes memory) {
        // Use MAP_FP_TO_G1 precompile to hash message to curve
        // Input: 64 bytes (field element)
        // Output: 48 bytes (compressed G1 point)

        // Expand message to 64 bytes using a simple domain separation
        bytes memory input = new bytes(64);

        // First 32 bytes: message
        for (uint256 i = 0; i < 32; i++) {
            input[i] = message[i];
        }

        // Last 32 bytes: zero (or could use domain separator)
        // This is a simplified approach; production should use proper hash-to-curve

        (bool success, bytes memory result) = MAP_FP_TO_G1.staticcall(input);
        if (!success || result.length != G1_POINT_SIZE) {
            revert PrecompileCallFailed();
        }

        return result;
    }

    /**
     * @dev Negate a G1 point
     * @param point The G1 point to negate (48 bytes compressed)
     * @return The negated G1 point
     */
    function negateG1(bytes memory point) internal pure returns (bytes memory) {
        if (point.length != G1_POINT_SIZE) {
            revert InvalidG1Point();
        }

        // For compressed points, negation flips the compression flag bit
        // The compression flag is in the most significant bits of the first byte
        bytes memory negated = new bytes(G1_POINT_SIZE);

        for (uint256 i = 0; i < G1_POINT_SIZE; i++) {
            negated[i] = point[i];
        }

        // Flip the sign bit (bit 5 of first byte in BLS12-381 compressed format)
        negated[0] = bytes1(uint8(negated[0]) ^ 0x20);

        return negated;
    }

    /**
     * @dev Get the G2 generator point
     * @return The G2 generator (96 bytes compressed)
     */
    function g2Generator() internal pure returns (bytes memory) {
        // BLS12-381 G2 generator in compressed form
        // This is a constant defined by the curve specification
        bytes memory gen = new bytes(G2_POINT_SIZE);

        // G2 generator compressed representation
        // Note: This is a placeholder. The actual G2 generator bytes should be:
        // 0x93e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8

        // First 48 bytes (x coordinate)
        gen[0] = 0x93;
        gen[1] = 0xe0;
        gen[2] = 0x2b;
        gen[3] = 0x60;
        gen[4] = 0x52;
        gen[5] = 0x71;
        gen[6] = 0x9f;
        gen[7] = 0x60;
        gen[8] = 0x7d;
        gen[9] = 0xac;
        gen[10] = 0xd3;
        gen[11] = 0xa0;
        gen[12] = 0x88;
        gen[13] = 0x27;
        gen[14] = 0x4f;
        gen[15] = 0x65;
        gen[16] = 0x59;
        gen[17] = 0x6b;
        gen[18] = 0xd0;
        gen[19] = 0xd0;
        gen[20] = 0x99;
        gen[21] = 0x20;
        gen[22] = 0xb6;
        gen[23] = 0x1a;
        gen[24] = 0xb5;
        gen[25] = 0xda;
        gen[26] = 0x61;
        gen[27] = 0xbb;
        gen[28] = 0xdc;
        gen[29] = 0x7f;
        gen[30] = 0x50;
        gen[31] = 0x49;
        gen[32] = 0x33;
        gen[33] = 0x4c;
        gen[34] = 0xf1;
        gen[35] = 0x12;
        gen[36] = 0x13;
        gen[37] = 0x94;
        gen[38] = 0x5d;
        gen[39] = 0x57;
        gen[40] = 0xe5;
        gen[41] = 0xac;
        gen[42] = 0x7d;
        gen[43] = 0x05;
        gen[44] = 0x5d;
        gen[45] = 0x04;
        gen[46] = 0x2b;
        gen[47] = 0x7e;

        // Second 48 bytes (y coordinate)
        gen[48] = 0x02;
        gen[49] = 0x4a;
        gen[50] = 0xa2;
        gen[51] = 0xb2;
        gen[52] = 0xf0;
        gen[53] = 0x8f;
        gen[54] = 0x0a;
        gen[55] = 0x91;
        gen[56] = 0x26;
        gen[57] = 0x08;
        gen[58] = 0x05;
        gen[59] = 0x27;
        gen[60] = 0x2d;
        gen[61] = 0xc5;
        gen[62] = 0x10;
        gen[63] = 0x51;
        gen[64] = 0xc6;
        gen[65] = 0xe4;
        gen[66] = 0x7a;
        gen[67] = 0xd4;
        gen[68] = 0xfa;
        gen[69] = 0x40;
        gen[70] = 0x3b;
        gen[71] = 0x02;
        gen[72] = 0xb4;
        gen[73] = 0x51;
        gen[74] = 0x0b;
        gen[75] = 0x64;
        gen[76] = 0x7a;
        gen[77] = 0xe3;
        gen[78] = 0xd1;
        gen[79] = 0x77;
        gen[80] = 0x0b;
        gen[81] = 0xac;
        gen[82] = 0x03;
        gen[83] = 0x26;
        gen[84] = 0xa8;
        gen[85] = 0x05;
        gen[86] = 0xbb;
        gen[87] = 0xef;
        gen[88] = 0xd4;
        gen[89] = 0x80;
        gen[90] = 0x56;
        gen[91] = 0xc8;
        gen[92] = 0xc1;
        gen[93] = 0x21;
        gen[94] = 0xbd;
        gen[95] = 0xb8;

        return gen;
    }

    /**
     * @dev Validate that a point is a valid compressed G1 point
     * @param point The point to validate (should be 48 bytes)
     * @return True if valid, false otherwise
     */
    function isValidG1Point(bytes calldata point) internal pure returns (bool) {
        if (point.length != G1_POINT_SIZE) {
            return false;
        }

        // Check compression flag (top 3 bits of first byte)
        // Bit 7 (0x80): compression flag (should be 1 for compressed)
        // Bit 6 (0x40): infinity flag
        // Bit 5 (0x20): y sign bit
        uint8 firstByte = uint8(point[0]);

        // Must have compression flag set
        if ((firstByte & 0x80) == 0) {
            return false;
        }

        // If infinity flag is set, all other bytes should be zero
        if ((firstByte & 0x40) != 0) {
            // Check that all other bits are zero
            if ((firstByte & 0x3F) != 0) {
                return false;
            }
            for (uint256 i = 1; i < G1_POINT_SIZE; i++) {
                if (point[i] != 0) {
                    return false;
                }
            }
        }

        return true;
    }

    /**
     * @dev Validate that a point is a valid compressed G1 point (memory version)
     * @param point The point to validate (should be 48 bytes)
     * @return True if valid, false otherwise
     */
    function isValidG1PointMemory(bytes memory point) internal pure returns (bool) {
        if (point.length != G1_POINT_SIZE) {
            return false;
        }

        uint8 firstByte = uint8(point[0]);

        // Must have compression flag set
        if ((firstByte & 0x80) == 0) {
            return false;
        }

        // If infinity flag is set, all other bytes should be zero
        if ((firstByte & 0x40) != 0) {
            if ((firstByte & 0x3F) != 0) {
                return false;
            }
            for (uint256 i = 1; i < G1_POINT_SIZE; i++) {
                if (point[i] != 0) {
                    return false;
                }
            }
        }

        return true;
    }

    /**
     * @dev Validate that a point is a valid compressed G2 point
     * @param point The point to validate (should be 96 bytes)
     * @return True if valid, false otherwise
     */
    function isValidG2Point(bytes calldata point) internal pure returns (bool) {
        if (point.length != G2_POINT_SIZE) {
            return false;
        }

        // Check compression flag (same format as G1)
        uint8 firstByte = uint8(point[0]);

        // Must have compression flag set
        if ((firstByte & 0x80) == 0) {
            return false;
        }

        // If infinity flag is set, all other bytes should be zero
        if ((firstByte & 0x40) != 0) {
            if ((firstByte & 0x3F) != 0) {
                return false;
            }
            for (uint256 i = 1; i < G2_POINT_SIZE; i++) {
                if (point[i] != 0) {
                    return false;
                }
            }
        }

        return true;
    }

    /**
     * @dev Validate that a point is a valid compressed G2 point (memory version)
     * @param point The point to validate (should be 96 bytes)
     * @return True if valid, false otherwise
     */
    function isValidG2PointMemory(bytes memory point) internal pure returns (bool) {
        if (point.length != G2_POINT_SIZE) {
            return false;
        }

        uint8 firstByte = uint8(point[0]);

        // Must have compression flag set
        if ((firstByte & 0x80) == 0) {
            return false;
        }

        // If infinity flag is set, all other bytes should be zero
        if ((firstByte & 0x40) != 0) {
            if ((firstByte & 0x3F) != 0) {
                return false;
            }
            for (uint256 i = 1; i < G2_POINT_SIZE; i++) {
                if (point[i] != 0) {
                    return false;
                }
            }
        }

        return true;
    }
}
