// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// End-to-end production replay for the SubstrateMerkleProof aliasing fix.
//
// Forks mainnet at the block just before each real transaction, replaces the live BeefyClient's
// CODE with the PATCHED build via vm.etch (real storage — validator set, tickets, prevRandao — is
// preserved), and replays the exact production calldata. Both must still succeed under the fixed
// code, confirming no regression on the real interactive flow.
//   submitInitial: 0xaf3a30dfa2ae6acb0217e50fedfbd864401101c747d5df7521ca9a4b3b946c2b (block 25236612)
//   submitFinal:   0xe2976fbbb5ae2e8f449ebf2b37576165f32a833e39befd190038690c05154e22 (block 25236743)
//   BeefyClient 0x7cfc5C8b341991993080Af67D940B6aD19a010E1; both from relayer 0xBa9b...Ed49.
//
// Needs an archive RPC for the fork. Run:
//   forge test --match-path test/SubstrateMerkleProofProdReplay.t.sol -vvv

import {Test} from "forge-std/Test.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

contract SubstrateMerkleProofProdReplayTest is Test {
    string constant RPC = "https://eth-mainnet.public.blastapi.io"; // archive
    address constant BC = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    address constant RELAYER = 0xBa9bC9a8Aa87872f7B990031bde984A00b9CEd49;

    uint256 constant SUBMIT_INITIAL_BLOCK = 25236612;
    uint256 constant SUBMIT_FINAL_BLOCK = 25236743;

    // Replace the live contract's code with a freshly-compiled PATCHED BeefyClient that carries the
    // same immutables (read from the live contract), preserving the live storage layout/state.
    function _etchPatched() internal {
        BeefyClient live = BeefyClient(BC);
        uint256 delay = live.randaoCommitDelay();
        uint256 expiry = live.randaoCommitExpiration();
        uint256 minSigs = live.minNumRequiredSignatures();
        uint256 fsSigs = live.fiatShamirRequiredSignatures();

        // Dummy sets only satisfy the constructor; we keep only the runtime CODE, not its storage.
        BeefyClient.ValidatorSet memory d0 = BeefyClient.ValidatorSet({id: 0, length: 1, root: bytes32(0)});
        BeefyClient.ValidatorSet memory d1 = BeefyClient.ValidatorSet({id: 1, length: 1, root: bytes32(0)});
        BeefyClient patched = new BeefyClient(delay, expiry, minSigs, fsSigs, 0, d0, d1);

        vm.etch(BC, address(patched).code);
        // Sanity: the etched patched code reads the live immutables/state unchanged.
        assertEq(BeefyClient(BC).randaoCommitDelay(), delay, "immutables preserved after etch");
    }

    function _revertReason(bytes memory ret) internal pure returns (string memory) {
        if (ret.length == 0) return "(no return data)";
        return vm.toString(ret);
    }

    function test_replay_submitInitial_onPatched() public {
        vm.createSelectFork(RPC, SUBMIT_INITIAL_BLOCK - 1);
        _etchPatched();
        vm.roll(SUBMIT_INITIAL_BLOCK);

        bytes memory cd = vm.parseBytes(vm.readFile("test/data/prod_submitInitial.hex"));
        vm.prank(RELAYER);
        (bool ok, bytes memory ret) = BC.call(cd);
        assertTrue(ok, string.concat("submitInitial must succeed on patched code: ", _revertReason(ret)));
    }

    function test_replay_submitFinal_onPatched() public {
        // Fork just before submitFinal: the ticket from the real submitInitial and the committed
        // prevRandao are already in storage, so the full flow's state is present.
        vm.createSelectFork(RPC, SUBMIT_FINAL_BLOCK - 1);
        _etchPatched();
        vm.roll(SUBMIT_FINAL_BLOCK);

        bytes32 rootBefore = BeefyClient(BC).latestMMRRoot();

        bytes memory cd = vm.parseBytes(vm.readFile("test/data/prod_submitFinal.hex"));
        vm.prank(RELAYER);
        (bool ok, bytes memory ret) = BC.call(cd);
        assertTrue(ok, string.concat("submitFinal must succeed on patched code: ", _revertReason(ret)));

        bytes32 rootAfter = BeefyClient(BC).latestMMRRoot();
        assertTrue(rootAfter != rootBefore, "submitFinal must update latestMMRRoot");
        emit log_named_bytes32("latestMMRRoot after replay", rootAfter);
    }
}
