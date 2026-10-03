// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Fork-mainnet replay of real BEEFY submissions (see `MainnetBeefyFixture`) through the
// multiproof BeefyClient. For each, fork one block before the tx, write the local BeefyClient
// over the live code (keeping its storage: the ticket, the validator sets), re-encode the
// proofs as a multiproof and replay the call from the original relayer. It must succeed and
// advance the MMR root.
//
// Run (needs an archive RPC; a public default is used if MAINNET_RPC_URL is unset):
//   FOUNDRY_PROFILE=integration forge test --match-contract ForkBeefyMultiproof -vv

import {BeefyClient} from "../src/BeefyClient.sol";
import {CompactProofLib} from "../test/utils/CompactProofLib.sol";
import {MainnetBeefyFixture} from "../test/MainnetSubmitFinalMultiproof.t.sol";

contract ForkBeefyMultiproofTest is MainnetBeefyFixture {
    function testMainnetSubmitFinalSucceedsAfterMultiproofEtch() public {
        _replay(finalE8eb06());
    }

    function testMainnetSubmitFinal992ebbSucceedsAfterMultiproofEtch() public {
        _replay(final992ebb());
    }

    function testMainnetSubmitFiatShamirSucceedsAfterMultiproofEtch() public {
        _replay(fiatShamir0a9f5a());
    }

    function _replay(MainnetTx memory t) internal {
        // Default is a public archive endpoint; override with MAINNET_RPC_URL.
        string memory rpc = vm.envOr("MAINNET_RPC_URL", string("https://eth.drpc.org"));
        vm.createSelectFork(rpc, t.blockNumber - 1);

        (uint128 id, uint128 len, bytes32 root,) =
            t.handover ? BeefyClient(BC).nextValidatorSet() : BeefyClient(BC).currentValidatorSet();
        assertEq(id, t.vsetId, "set id");
        assertEq(len, t.vsetLength, "set length");
        assertEq(root, t.vsetRoot, "set root");

        (bytes memory multiproofCd, uint64 beefyBlock) = _multiproofCalldata(t);

        bytes32 mmrBefore = BeefyClient(BC).latestMMRRoot();
        _etchMultiproof();
        vm.roll(t.blockNumber);
        vm.prank(RELAYER);
        (bool ok, bytes memory ret) = BC.call(multiproofCd);
        assertTrue(ok, string.concat(t.name, " reverted: ", vm.toString(ret)));

        assertEq(BeefyClient(BC).latestBeefyBlock(), beefyBlock, "beefy block");
        assertTrue(BeefyClient(BC).latestMMRRoot() != mmrBefore, "MMR root must advance");
    }

    /// The fixture's call with the proofs re-encoded as a multiproof.
    function _multiproofCalldata(MainnetTx memory t)
        internal
        view
        returns (bytes memory cd, uint64 beefyBlock)
    {
        (
            BeefyClient.Commitment memory commitment,
            uint256[] memory bitfield,
            BeefyClient.ValidatorProof[] memory proofs,
            BeefyClient.MMRLeaf memory leaf,
            bytes32[] memory leafProof,
            uint256 leafProofOrder
        ) = _load(t);
        cd = abi.encodeWithSelector(
            t.fiatShamir
                ? BeefyClient.submitFiatShamir.selector
                : BeefyClient.submitFinal.selector,
            commitment,
            bitfield,
            CompactProofLib.toCompact(proofs, t.vsetLength),
            leaf,
            leafProof,
            leafProofOrder
        );
        beefyBlock = commitment.blockNumber;
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
