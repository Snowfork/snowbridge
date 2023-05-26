// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "openzeppelin/utils/Strings.sol";
import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../src/SubstrateTypes.sol";
import "../src/ScaleCodec.sol";

enum DigestItemKind {
    PreRuntime,
    Consensus,
    Seal,
    Other,
    RuntimeEnvironmentUpdated
}

struct DigestItem {
    DigestItemKind kind;
    bytes4 consensusEngineID;
    bytes data;
}

struct ParachainHeader {
    bytes32 parentHash;
    uint256 number;
    bytes32 stateRoot;
    bytes32 extrinsicsRoot;
    DigestItem[] digestItems;
}

contract DigestItemTest is Test {
    function join(bytes[] memory a, bytes memory glue) public pure returns (bytes memory) {
        uint256 inputPointer;
        uint256 gluePointer;

        assembly {
            inputPointer := a
            gluePointer := glue
        }
        return _joinReferenceType(inputPointer, gluePointer);
    }

    function join(bytes[] memory a) public pure returns (bytes memory) {
        return join(a, bytes(""));
    }

    // Copyright 2022 Clement Walter
    // https://github.com/ClementWalter/eth-projects-monorepo/blob/main/packages/eth-projects-contracts/contracts/lib/utils/Array.sol
    function _joinReferenceType(uint256 inputPointer, uint256 gluePointer)
        public
        pure
        returns (bytes memory tempBytes)
    {
        assembly {
            // Get a location of some free memory and store it in tempBytes as
            // Solidity does for memory variables.
            tempBytes := mload(0x40)

            // Skip the first 32 bytes where we will store the length of the result
            let memoryPointer := add(tempBytes, 0x20)

            // Load glue
            let glueLength := mload(gluePointer)
            if gt(glueLength, 0x20) { revert(gluePointer, 0x20) }
            let glue := mload(add(gluePointer, 0x20))

            // Load the length (first 32 bytes)
            let inputLength := mload(inputPointer)
            let inputData := add(inputPointer, 0x20)
            let end := add(inputData, mul(inputLength, 0x20))

            // Initialize the length of the final string
            let stringLength := 0

            // Iterate over all strings (a string is itself an array).
            for { let pointer := inputData } lt(pointer, end) { pointer := add(pointer, 0x20) } {
                let currentStringArray := mload(pointer)
                let currentStringLength := mload(currentStringArray)
                stringLength := add(stringLength, currentStringLength)
                let currentStringBytesCount :=
                    add(div(currentStringLength, 0x20), gt(mod(currentStringLength, 0x20), 0))

                let currentPointer := add(currentStringArray, 0x20)

                for { let copiedBytesCount := 0 } lt(copiedBytesCount, currentStringBytesCount) {
                    copiedBytesCount := add(copiedBytesCount, 1)
                } {
                    mstore(add(memoryPointer, mul(copiedBytesCount, 0x20)), mload(currentPointer))
                    currentPointer := add(currentPointer, 0x20)
                }
                memoryPointer := add(memoryPointer, currentStringLength)
                mstore(memoryPointer, glue)
                memoryPointer := add(memoryPointer, glueLength)
            }

            mstore(tempBytes, add(stringLength, mul(sub(inputLength, 1), glueLength)))
            mstore(0x40, and(add(memoryPointer, 31), not(31)))
        }
        return tempBytes;
    }

    function sizeOfEncodedCompactUint(uint256 value) internal pure returns (uint256) {
        if (value <= 2 ** 6 - 1) {
            return 1;
        } else if (value <= 2 ** 14 - 1) {
            return 2;
        } else {
            return 4;
        }
    }

    // function sizeOfDigestItems(DigestItem[] memory digestItems) internal pure returns (uint256) {
    //     uint256 accum = 0;
    //     for (uint256 i = 0; i < digestItems.length; i++) {
    //         DigestItem memory digestItem = digestItems[i];
    //         if (digestItem.kind == DigestItemKind.PreRuntime) {
    //             accum += 1 + 4 + sizeOfEncodedCompactUint(digestItem.data.length) + digestItem.data.length;
    //         } else if (digestItem.kind == DigestItemKind.Consensus) {
    //             accum += 1 + 4 + sizeOfEncodedCompactUint(digestItem.data.length) + digestItem.data.length;
    //         } else if (digestItem.kind == DigestItemKind.Seal) {
    //             accum += 1 + 4 + sizeOfEncodedCompactUint(digestItem.data.length) + digestItem.data.length;
    //         } else if (digestItem.kind == DigestItemKind.Other) {
    //             accum += 1 + sizeOfEncodedCompactUint(digestItem.data.length) + digestItem.data.length;
    //         } else {
    //             accum += 1;
    //         }
    //     }
    // }

    function encodeDigestItem(DigestItem memory digestItem) internal pure returns (bytes memory) {
        if (digestItem.kind == DigestItemKind.PreRuntime) {
            return bytes.concat(hex"00", digestItem.consensusEngineID, SubstrateTypes.VecU8(digestItem.data));
        } else if (digestItem.kind == DigestItemKind.Consensus) {
            return bytes.concat(hex"01", digestItem.consensusEngineID, SubstrateTypes.VecU8(digestItem.data));
        } else if (digestItem.kind == DigestItemKind.Seal) {
            return bytes.concat(hex"02", digestItem.consensusEngineID, SubstrateTypes.VecU8(digestItem.data));
        } else if (digestItem.kind == DigestItemKind.Other) {
            return bytes.concat(hex"03", SubstrateTypes.VecU8(digestItem.data));
        } else {
            return bytes(hex"04");
        }
    }

    function encodeParachainHeader(ParachainHeader memory header) internal pure returns (bytes32) {
        bytes[] memory encodedDigestItems = new bytes[](header.digestItems.length);
        for (uint256 i = 0; i < header.digestItems.length; i++) {
            encodedDigestItems[i] = encodeDigestItem(header.digestItems[i]);
        }

        bytes memory encodedDigestItems2 = join(encodedDigestItems);
        bytes memory encodedDigestItems3 =
            bytes.concat(ScaleCodec.encodeCompactUint(header.digestItems.length), encodedDigestItems2);

        return keccak256(
            bytes.concat(
                header.parentHash,
                ScaleCodec.encodeCompactUint(header.number),
                header.stateRoot,
                header.extrinsicsRoot,
                encodedDigestItems3
            )
        );
    }

    function testEncodeParachainHeader() public view {
        DigestItem[] memory items = new DigestItem[](3);
        items[0] = DigestItem(
            DigestItemKind.PreRuntime,
            hex"61757261",
            bytes("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c")
        );
        items[1] = DigestItem(
            DigestItemKind.Consensus,
            hex"61757261",
            bytes("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c")
        );
        items[2] = DigestItem(
            DigestItemKind.Seal,
            hex"61757261",
            bytes("b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c")
        );

        ParachainHeader memory header = ParachainHeader({
            parentHash: bytes32(hex"b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c"),
            number: 3,
            stateRoot: bytes32(hex"b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c"),
            extrinsicsRoot: bytes32(hex"b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c"),
            digestItems: items
        });

        uint256 x = gasleft();
        encodeParachainHeader(header);
        uint256 y = gasleft();

        console.log("gas used: %d", x - y);
    }
}
