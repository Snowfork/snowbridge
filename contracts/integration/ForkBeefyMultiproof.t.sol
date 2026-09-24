// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Fork-mainnet replay of a REAL production `submitFinal` through the multiproof BeefyClient.
//
// Source transaction:
//   0xe8eb06c8e18879418408e255b0fd0e9cc72b9c668638f324735b98a0cb045936  (block 26,046,024)
// It carries 28 legacy `ValidatorProof`s signed by validator set 5688, the *next* set at the
// fork block, so it also covers the handover path. We fork one block BEFORE the tx, write the
// locally-compiled BeefyClient over the live contract's code (keeping its storage: the ticket,
// the validator sets), reassemble the proofs into the multiproof format, and replay the call
// from the original relayer. It must succeed and advance the MMR root.
//
// Run (needs an archive RPC; a public default is used if MAINNET_RPC_URL is unset):
//   FOUNDRY_PROFILE=integration forge test --match-contract ForkBeefyMultiproof -vv

import {BeefyClient} from "../src/BeefyClient.sol";
import {CompactProofLib} from "../test/utils/CompactProofLib.sol";
import {MainnetSubmitFinalFixture} from "../test/MainnetSubmitFinalMultiproof.t.sol";

contract ForkBeefyMultiproofTest is MainnetSubmitFinalFixture {
    function testMainnetSubmitFinalSucceedsAfterMultiproofEtch() public {
        // Default is a public archive endpoint; override with MAINNET_RPC_URL.
        string memory rpc = vm.envOr("MAINNET_RPC_URL", string("https://eth.drpc.org"));
        vm.createSelectFork(rpc, FINAL_BLOCK - 1);

        (
            BeefyClient.Commitment memory commitment,
            uint256[] memory bitfield,
            BeefyClient.ValidatorProof[] memory proofs,
            BeefyClient.MMRLeaf memory leaf,
            bytes32[] memory leafProof,
            uint256 leafProofOrder
        ) = _load();

        (uint128 nid, uint128 nlen, bytes32 nroot,) = BeefyClient(BC).nextValidatorSet();
        assertEq(nid, VSET_ID, "next set id");
        assertEq(nlen, VSET_LENGTH, "next set length");
        assertEq(nroot, VSET_ROOT, "next set root");

        bytes memory multiproofCd = abi.encodeWithSelector(
            BeefyClient.submitFinal.selector,
            commitment,
            bitfield,
            CompactProofLib.toCompact(proofs, VSET_LENGTH),
            leaf,
            leafProof,
            leafProofOrder
        );

        bytes32 mmrBefore = BeefyClient(BC).latestMMRRoot();
        _etchMultiproof();
        vm.roll(FINAL_BLOCK);
        vm.prank(RELAYER);
        (bool ok, bytes memory ret) = BC.call(multiproofCd);
        assertTrue(ok, string.concat("submitFinal reverted: ", vm.toString(ret)));

        assertEq(BeefyClient(BC).latestBeefyBlock(), commitment.blockNumber, "beefy block");
        assertTrue(BeefyClient(BC).latestMMRRoot() != mmrBefore, "MMR root must advance");
    }

    function _etchMultiproof() internal {
        BeefyClient live = BeefyClient(BC);
        BeefyClient.ValidatorSet memory d0 =
            BeefyClient.ValidatorSet({id: 0, length: 1, root: bytes32(0)});
        BeefyClient.ValidatorSet memory d1 =
            BeefyClient.ValidatorSet({id: 1, length: 1, root: bytes32(0)});
        // Immutables live in the code, so rebuild it with the live values.
        BeefyClient patched = new BeefyClient(
            live.randaoCommitDelay(),
            live.randaoCommitExpiration(),
            live.minNumRequiredSignatures(),
            live.fiatShamirRequiredSignatures(),
            0,
            d0,
            d1
        );
        vm.etch(BC, address(patched).code);
    }
}
