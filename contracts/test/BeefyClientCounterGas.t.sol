// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientTest} from "./BeefyClient.t.sol";

/// @dev Gas of the interactive and Fiat-Shamir paths with the slots that are always non-zero on
/// mainnet (latestBeefyBlock, latestMMRRoot) made non-zero first.
///
/// FOUNDRY_PROFILE=production FOUNDRY_ISOLATE=true forge test \
///     --match-contract '^BeefyClientCounterGasTest$' --hardfork prague --gas-report
contract BeefyClientCounterGasTest is BeefyClientTest {
    function warm(uint32 currentSetId)
        internal
        returns (BeefyClient.Commitment memory commitment)
    {
        commitment = initialize(currentSetId);
        beefyClient.setLatestBeefyBlock(1);
        beefyClient.setLatestMMRRoot(bytes32(uint256(1)));
    }

    function run(BeefyClient.Commitment memory commitment) internal {
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testGasSameSet() public {
        run(warm(setId));
    }

    function testGasHandover() public {
        run(warm(setId - 1));
    }

    function runFiatShamir(BeefyClient.Commitment memory commitment) internal {
        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testGasFiatShamirSameSet() public {
        runFiatShamir(warm(setId));
    }

    function testGasFiatShamirHandover() public {
        runFiatShamir(warm(setId - 1));
    }
}
