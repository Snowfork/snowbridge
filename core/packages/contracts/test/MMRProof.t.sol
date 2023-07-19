// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {MMRProof} from "../src/utils/MMRProof.sol";
import {MMRProofWrapper} from "./mocks/MMRProofWrapper.sol";

contract MMRProofTest is Test {
    using stdJson for string;

    struct Fixture {
        bytes32[] leaves;
        Proof[] proofs;
        bytes32 rootHash;
    }

    struct Proof {
        bytes32[] items;
        uint256 order;
    }

    Fixture public fixture;

    MMRProofWrapper public wrapper;

    function setUp() public {
        wrapper = new MMRProofWrapper();

        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/test/data/mmr-fixture-data-15-leaves.json");
        string memory json = vm.readFile(path);
        bytes memory mockData = json.parseRaw("");
        fixture = abi.decode(mockData, (Fixture));
    }

    function testVerifyLeafProof() public {
        for (uint256 i = 0; i < fixture.leaves.length; i++) {
            assertTrue(
                wrapper.verifyLeafProof(
                    fixture.rootHash, fixture.leaves[i], fixture.proofs[i].items, fixture.proofs[i].order
                )
            );
        }
    }

    function testVerifyLeafProofFailsExceededProofSize() public {
        vm.expectRevert(MMRProof.ProofSizeExceeded.selector);
        wrapper.verifyLeafProof(fixture.rootHash, fixture.leaves[0], new bytes32[](257), fixture.proofs[0].order);
    }
}
