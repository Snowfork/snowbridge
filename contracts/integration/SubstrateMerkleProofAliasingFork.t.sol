// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "../test/mocks/BeefyClientMock.sol";
import {BeefyClientForgeRejectionTest} from "../test/SubstrateMerkleProofAliasing.t.sol";

contract SubstrateMerkleProofAliasingForkTest is BeefyClientForgeRejectionTest {
    function test_fix_forgeRejectedAtLiveValidatorCount() public {
        vm.createSelectFork("https://ethereum-rpc.publicnode.com");
        (uint128 id, uint128 len,,) = BeefyClient(LIVE).currentValidatorSet();
        assertEq(len, N, "live validator count read from mainnet is 600");

        (BeefyClientMock bc, uint256[] memory bf, BeefyClient.ValidatorProof[] memory proofs) =
            _setupInteractive(uint64(id));

        BeefyClient.MMRLeaf memory leaf;
        bytes32[] memory empty = new bytes32[](0);
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        bc.submitFinal(_commit(), bf, proofs, leaf, empty, 0);

        assertTrue(
            bc.latestMMRRoot() != forged, "patched contract at live count rejects the forge"
        );
    }
}
