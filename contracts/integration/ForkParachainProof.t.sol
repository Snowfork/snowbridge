// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Fork-mainnet replay of a REAL production parachain-heads proof through the FIXED
// SubstrateMerkleProof / Verification code.
//
// Source transaction:
//   0xe57890eec44787cad782fbe6ae67edd5fc8f6a370b88a674f000f295aef0a96a  (block 25,326,587)
// It is a Multicall3 aggregate3 with two sub-calls:
//   call[0] -> BeefyClient.submitFiatShamir(...)   (BEEFY consensus update; advances latestMMRRoot)
//   call[1] -> Gateway.submitV1(message, leafProof, headerProof)   (the transfer message)
//
// `headerProof` in call[1] carries the parachain-heads merkle proof that flows through
// SubstrateMerkleProof.computeRoot (the function changed in PR #24). We fork AFTER the tx so the
// live BeefyClient already holds the MMR root that call[0] committed, decode call[1]'s real
// arguments, recompute the message commitment exactly as Gateway.submitV1 does, then run the
// locally-compiled (i.e. FIXED) Verification.verifyCommitment against the live BeefyClient.
//
//   - LIVENESS: the fixed library must ACCEPT the real, canonical production proof.
//   - SECURITY: zero-padding that same real proof must be REJECTED by the exact-consumption check.
//
// Run (needs an archive RPC; a public default is used if MAINNET_RPC_URL is unset):
//   FOUNDRY_PROFILE=integration forge test --match-contract ForkParachainProof -vv

import {Test} from "forge-std/Test.sol";
import {Verification} from "../src/Verification.sol";
import {InboundMessage} from "../src/v1/Types.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";

// Bridges a memory-built proof to the calldata parameter of the library entrypoint, so the test
// exercises the real `Verification.verifyCommitment` ABI path.
contract VerifyHarness {
    function verify(
        address beefyClient,
        bytes4 encodedParaID,
        bytes32 commitment,
        Verification.Proof calldata proof,
        bool isV2
    ) external view returns (bool) {
        return Verification.verifyCommitment(beefyClient, encodedParaID, commitment, proof, isV2);
    }
}

contract ForkParachainProofTest is Test {
    // Mainnet deployments at the fork block.
    address constant BEEFY_CLIENT = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    // BridgeHub para id on Polkadot (src/Constants.sol BRIDGE_HUB_PARA_ID).
    uint32 constant BRIDGE_HUB_PARA_ID = 1002;
    // One block AFTER the source tx, so call[0]'s MMR root update is in state.
    uint256 constant FORK_BLOCK = 25_326_588;

    VerifyHarness harness;
    bytes4 encodedParaID;

    function setUp() public {
        // Default is a public archive endpoint; override with MAINNET_RPC_URL (the public default
        // may rate-limit or prune over time, so CI should set its own archive RPC).
        string memory rpc = vm.envOr("MAINNET_RPC_URL", string("https://eth.drpc.org"));
        vm.createSelectFork(rpc, FORK_BLOCK);

        harness = new VerifyHarness();
        encodedParaID = ScaleCodec.encodeU32(BRIDGE_HUB_PARA_ID);
    }

    // Decode the real submitV1 arguments captured from the source tx and reproduce
    // Gateway.submitV1's commitment derivation. Returned in memory (nested dynamic structs cannot
    // live in storage on the legacy pipeline).
    function _load()
        internal
        view
        returns (Verification.Proof memory headerProof, bytes32 commitment)
    {
        bytes memory call1 = vm.parseBytes(vm.readFile("test/data/gateway-submitv1-call.hex"));
        bytes memory args = _stripSelector(call1);
        InboundMessage memory message;
        bytes32[] memory leafProof;
        (message, leafProof, headerProof) =
            abi.decode(args, (InboundMessage, bytes32[], Verification.Proof));

        commitment = MerkleProof.processProof(leafProof, keccak256(abi.encode(message)));
    }

    // LIVENESS: the fixed library accepts the real production parachain-heads proof, verified all
    // the way to the live BeefyClient MMR root.
    function testFixedLibraryAcceptsRealProductionProof() public view {
        (Verification.Proof memory headerProof, bytes32 commitment) = _load();
        bool ok =
            harness.verify(BEEFY_CLIENT, encodedParaID, commitment, headerProof, false);
        assertTrue(ok, "fixed Verification must accept the real mainnet parachain-heads proof");
    }

    // SECURITY: zero-padding the real (canonical) proof is rejected by exact-consumption.
    function testPaddedRealProofRejected() public view {
        (Verification.Proof memory headerProof, bytes32 commitment) = _load();
        bytes32[] memory full = headerProof.headProof.proof;
        bytes32[] memory padded = new bytes32[](full.length + 1);
        for (uint256 i = 0; i < full.length; i++) {
            padded[i] = full[i];
        }
        padded[full.length] = bytes32(0);
        headerProof.headProof.proof = padded;

        bool ok =
            harness.verify(BEEFY_CLIENT, encodedParaID, commitment, headerProof, false);
        assertFalse(ok, "padded parachain-heads proof must be rejected on real data");
    }

    function _stripSelector(bytes memory data) internal pure returns (bytes memory out) {
        require(data.length >= 4, "calldata too short");
        out = new bytes(data.length - 4);
        for (uint256 i = 0; i < out.length; i++) {
            out[i] = data[i + 4];
        }
    }
}
