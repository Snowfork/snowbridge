// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

// `signatureAt` reads via inline assembly at `signatures.offset + i*65`. Its safety rests
// entirely on (a) the length check `signatures.length == k*65` and (b) Solidity's ABI decoder
// guaranteeing the blob lies inside calldata. (b) is an assumption about the compiler, so
// attack it with hand-crafted calldata rather than trusting it.
contract Harness {
    event Read(uint8 v, bytes32 r, bytes32 s);

    function take(BeefyClient.CompactValidatorProofs calldata p, uint256 k)
        external
        pure
        returns (uint8 v, bytes32 r, bytes32 s, uint256 len)
    {
        require(p.signatures.length == k * 65, "len");
        (v, r, s) = sigAt(p.signatures, k - 1);
        len = p.signatures.length;
    }

    function sigAt(bytes calldata signatures, uint256 i)
        internal
        pure
        returns (uint8 v, bytes32 r, bytes32 s)
    {
        assembly {
            let o := add(signatures.offset, mul(i, 65))
            r := calldataload(o)
            s := calldataload(add(o, 32))
            v := byte(0, calldataload(add(o, 64)))
        }
    }
}

contract ZAuditCalldata is Test {
    Harness h;

    function setUp() public {
        h = new Harness();
    }

    // A signatures blob whose declared length runs past the end of calldata must be rejected
    // by the decoder, before any assembly runs.
    function testDecoderRejectsBlobLongerThanCalldata() public {
        // selector, struct offset, [sig offset, siblings offset], sig length = huge, no data
        bytes memory bad = abi.encodePacked(
            bytes4(keccak256("take((bytes,bytes32[]),uint256)")),
            uint256(0x40), // offset to struct
            uint256(1), // k
            uint256(0x40), // struct: offset to signatures
            uint256(0x80), // struct: offset to siblings
            uint256(65), // signatures.length = 65 ...
            bytes32(0) // ... but only 32 bytes follow
        );
        (bool ok,) = address(h).staticcall(bad);
        assertFalse(ok, "decoder accepted a blob that runs past calldatasize");
    }

    // A well-formed call: the last signature's `v` must be the byte at offset 64 of its record,
    // even though calldataload there also pulls in 31 bytes of whatever follows.
    function testLastSignatureVIsReadFromItsOwnRecord() public view {
        uint256 k = 3;
        bytes memory sigs;
        for (uint256 i = 0; i < k; i++) {
            sigs = bytes.concat(
                sigs, bytes32(uint256(0xAA00 + i)), bytes32(uint256(0xBB00 + i)), bytes1(uint8(27 + (i % 2)))
            );
        }
        bytes32[] memory sib = new bytes32[](2);
        sib[0] = bytes32(type(uint256).max); // adjacent data is all 0xff
        sib[1] = bytes32(type(uint256).max);

        (uint8 v, bytes32 r, bytes32 s, uint256 len) =
            h.take(BeefyClient.CompactValidatorProofs(sigs, sib), k);

        assertEq(len, k * 65);
        assertEq(uint256(r), 0xAA00 + (k - 1), "r of last record");
        assertEq(uint256(s), 0xBB00 + (k - 1), "s of last record");
        assertEq(v, 27 + ((k - 1) % 2), "v must come from the record, not adjacent calldata");
    }

    // Trailing ABI padding on the bytes field is attacker-controlled but must not be reachable.
    function testTrailingPaddingIsNotReadAsSignatureData() public view {
        uint256 k = 1;
        bytes memory sigs =
            bytes.concat(bytes32(uint256(1)), bytes32(uint256(2)), bytes1(uint8(28)));
        bytes32[] memory sib = new bytes32[](1);
        sib[0] = bytes32(type(uint256).max);
        (uint8 v,,,) = h.take(BeefyClient.CompactValidatorProofs(sigs, sib), k);
        assertEq(v, 28, "v polluted by padding or adjacent field");
    }
}
