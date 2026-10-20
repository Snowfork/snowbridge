// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

/// @dev `Head` packs the latest block and both validator sets' ids and lengths into one slot,
/// with narrower types than the constructor takes. Out-of-range values must be rejected, and
/// in-range values must read back unchanged.
contract BeefyClientHeadTest is Test {
    uint256 constant MAX32 = type(uint32).max;
    uint256 constant MAX64 = type(uint64).max;

    function deploy(
        uint256 initialBlock,
        uint256 id,
        uint256 length,
        uint256 nextLength,
        bytes32 root,
        bytes32 nextRoot
    ) internal returns (BeefyClient) {
        return new BeefyClient(
            3,
            8,
            16,
            111,
            uint64(initialBlock),
            // forge-lint: disable-next-line(unsafe-typecast)
            BeefyClient.ValidatorSet(uint128(id), uint128(length), root),
            // forge-lint: disable-next-line(unsafe-typecast)
            BeefyClient.ValidatorSet(uint128(id + 1), uint128(nextLength), nextRoot)
        );
    }

    function testExtremeValuesReadBackUnchanged() public {
        BeefyClient client =
            deploy(MAX32, MAX64 - 1, MAX32, MAX32 - 1, keccak256("root"), keccak256("next"));
        assertEq(client.latestBeefyBlock(), MAX32, "latest block");
        (uint128 id, uint128 length, bytes32 root) = client.currentValidatorSet();
        assertEq(id, MAX64 - 1, "current id");
        assertEq(length, MAX32, "current length");
        assertEq(root, keccak256("root"), "current root");
        (id, length, root) = client.nextValidatorSet();
        assertEq(id, MAX64, "next id");
        assertEq(length, MAX32 - 1, "next length");
        assertEq(root, keccak256("next"), "next root");
    }

    function testRejectsABlockPastUint32() public {
        vm.expectRevert(bytes("invalid-constructor-params"));
        deploy(MAX32 + 1, 1, 600, 600, 0, 0);
    }

    function testRejectsANextSetIdPastUint64() public {
        vm.expectRevert(bytes("invalid-constructor-params"));
        deploy(0, MAX64, 600, 600, 0, 0);
    }

    function testRejectsLengthsPastUint32() public {
        vm.expectRevert(bytes("invalid-constructor-params"));
        deploy(0, 1, MAX32 + 1, 600, 0, 0);
        vm.expectRevert(bytes("invalid-constructor-params"));
        deploy(0, 1, 600, MAX32 + 1, 0, 0);
    }
}
