pragma solidity ^0.8.28;
import "forge-std/Test.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";

contract FiatShamirPaddingBitsGrindTest is Test {
    using Bitfield for uint256[];
    uint256 internal constant N = 16;
    uint256 internal constant FIAT_REQUIRED = 6;
    BeefyClient internal beefy;
    uint256[] internal privKeys;
    address[] internal validators;
    bytes32[] internal leaves;
    bytes32 internal vsetRoot;

    function setUp() public {
        // 1) 16 validators with known keys (only keys [0..5] will be used by the "attacker")
        privKeys = new uint256[](N);
        validators = new address[](N);
        leaves = new bytes32[](N);
        for (uint256 i = 0; i < N; i++) {
            privKeys[i] = i + 1;
            validators[i] = vm.addr(privKeys[i]);
            leaves[i] = keccak256(abi.encodePacked(validators[i]));
        }
        // 2) a standard balanced merkle root (N=16 is power-of-two => proof gen is simple)
        vsetRoot = _merkleRootPow2(leaves);
        BeefyClient.ValidatorSet memory initial =
            BeefyClient.ValidatorSet({id: 1, length: uint128(N), root: vsetRoot});
        BeefyClient.ValidatorSet memory next =
            BeefyClient.ValidatorSet({id: 2, length: uint128(N), root: vsetRoot});
        beefy = new BeefyClient(0, 0, 1, FIAT_REQUIRED, 0, initial, next);
    }

    function test_exploit_grindPaddingBits_changesSample_and_allows_fraudulentMMRRootUpdate()
        public
    {
        // - attacker model:
        // - validator set length = 16
        // - quorum requires claiming >= 11 signatures (2/3+)
        // - fiat–Shamir requires proving 6 sampled signatures
        // - attacker ONLY has signatures from validators indices [0..5]
        // - they will "lie" in the bitfield and claim indices [6..10] also signed,
        // to satisfy quorum, but cannot actually provide those signatures
        // - the attack is to grind padding bits to make the sample land only on [0..5]
        // ----------------------------
        bytes32 maliciousMMRRoot = keccak256("attacker-chosen-mmr-root");
        BeefyClient.Commitment memory commitment =
            _commitment({beefyBlock: 1, vsetId: 1, mmrRoot: maliciousMMRRoot});
        bytes32 commitmentHash = keccak256(_encodeCommitment(commitment));
        // base bitfield meaning: claim bits [0..10] set (11 signers => quorum satisfied),
        // but bits [11..15] unset
        // padding bits [16..255] are "free nonce" and must NOT affect quorum/sampling semantics,
        // but DO affect fiat-shamir seed because they are hashed in abi.encodePacked(bitfield)
        uint256 baseMeaningful = (uint256(1) << 11) - 1; // bits 0..10 set
        uint256[] memory bitfield0 = new uint256[](1);
        bitfield0[0] = baseMeaningful;
        assertEq(bitfield0.countSetBits(N), 11, "quorum bits must be 11");
        uint256[] memory sampled0 = beefy.createFiatShamirFinalBitfield(commitment, bitfield0);
        // proofs ONLY for indices [0..5] (attacker-controlled keys)
        BeefyClient.ValidatorProof[] memory attackerProofs = _buildProofs(commitmentHash, 0, 6);
        // baseline should almost surely fail because sampled0 likely includes some index > 5
        // if by chance baseline matched (very low probability), keep flipping padding bits until it doesn't
        for (uint256 j = 0; j < 256 && _sampleIsExactlyFirstK(sampled0, 6); j++) {
            bitfield0[0] = baseMeaningful | (uint256(j + 1) << 16);
            sampled0 = beefy.createFiatShamirFinalBitfield(commitment, bitfield0);
        }
        assertTrue(
            !_sampleIsExactlyFirstK(sampled0, 6), "baseline unexpectedly matches attacker set"
        );
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefy.submitFiatShamir(
            commitment,
            bitfield0,
            attackerProofs,
            BeefyClient.MMRLeaf({
                version: 0,
                parentNumber: 0,
                parentHash: bytes32(0),
                nextAuthoritySetID: 0,
                nextAuthoritySetLen: 0,
                nextAuthoritySetRoot: bytes32(0),
                parachainHeadsRoot: bytes32(0)
            }),
            new bytes32[](0),
            0
        );
        // exploit: grind ONLY padding bits until sampled set becomes [0..5]
        (uint256 nonce, uint256[] memory grindedBitfield, uint256[] memory sampledGood) =
            _grindPaddingNonce(commitment, baseMeaningful, 6, 50_000);
        // verify: meaningful bits (0..15) unchanged
        uint256 meaningMask = (uint256(1) << N) - 1;
        assertEq(
            grindedBitfield[0] & meaningMask,
            baseMeaningful & meaningMask,
            "meaning changed unexpectedly"
        );
        // verify: bounded count identical, despite nonce
        assertEq(grindedBitfield.countSetBits(N), 11, "bounded count changed unexpectedly");
        // verify: but Fiat–Shamir sample is now attacker-friendly
        assertTrue(
            _sampleIsExactlyFirstK(sampledGood, 6),
            "grinding did not yield attacker-friendly sample"
        );
        // shows the root cause: hash changes due to padding bits
        bytes32 h0 = keccak256(abi.encodePacked(bitfield0));
        bytes32 h1 = keccak256(abi.encodePacked(grindedBitfield));
        assertTrue(h0 != h1, "padding nonce did not change bitfield hash");
        emit log_named_uint("Found padding nonce", nonce);
        emit log_named_bytes32("Baseline bitfieldHash", h0);
        emit log_named_bytes32("Grinded bitfieldHash", h1);
        // with the added validatePadding check, submission will revert for the attacker-controlled grindedBitfield signatures
        vm.expectRevert(Bitfield.InvalidBitfieldPadding.selector);
        beefy.submitFiatShamir(
            commitment,
            grindedBitfield,
            attackerProofs,
            BeefyClient.MMRLeaf({
                version: 0,
                parentNumber: 0,
                parentHash: bytes32(0),
                nextAuthoritySetID: 0,
                nextAuthoritySetLen: 0,
                nextAuthoritySetRoot: bytes32(0),
                parachainHeadsRoot: bytes32(0)
            }),
            new bytes32[](0),
            0
        );
    }

    // Grinding helper
    function _grindPaddingNonce(
        BeefyClient.Commitment memory commitment,
        uint256 baseMeaningfulLowBits,
        uint256 k,
        uint256 maxTries
    )
        internal
        view
        returns (uint256 nonceFound, uint256[] memory bitfieldFound, uint256[] memory sampled)
    {
        // padding starts at bit index N (16). Bits >= N do not affect bounded countSetBits(bitfield, N)
        for (uint256 nonce = 0; nonce < maxTries; nonce++) {
            uint256[] memory bf = new uint256[](1);
            bf[0] = baseMeaningfulLowBits | (nonce << N);
            uint256[] memory s = beefy.createFiatShamirFinalBitfield(commitment, bf);
            if (_sampleIsExactlyFirstK(s, k)) {
                return (nonce, bf, s);
            }
        }
        revert("grind: no nonce found within maxTries");
    }

    function _sampleIsExactlyFirstK(uint256[] memory sampledBitfield, uint256 k)
        internal
        pure
        returns (bool)
    {
        // sampledBitfield has exactly k bits set (because BeefyClient subsamples with n=k)
        for (uint256 i = 0; i < k; i++) {
            if (!sampledBitfield.isSet(i)) return false;
        }
        // if all first k are set, since there are exactly k set bits total, it must be exactly {0..k-1}
        return true;
    }

    // Validator proofs
    function _buildProofs(bytes32 commitmentHash, uint256 startIndex, uint256 count)
        internal
        returns (BeefyClient.ValidatorProof[] memory proofs)
    {
        proofs = new BeefyClient.ValidatorProof[](count);
        for (uint256 i = 0; i < count; i++) {
            uint256 validatorIndex = startIndex + i;
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(privKeys[validatorIndex], commitmentHash);
            bytes32[] memory merkleProof = _merkleProofPow2(leaves, validatorIndex);
            proofs[i] = BeefyClient.ValidatorProof({
                v: v,
                r: r,
                s: s,
                index: validatorIndex,
                account: validators[validatorIndex],
                proof: merkleProof
            });
        }
    }

    // Commitment encoding
    function _commitment(uint32 beefyBlock, uint64 vsetId, bytes32 mmrRoot)
        internal
        view
        returns (BeefyClient.Commitment memory c)
    {
        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem({
            payloadID: beefy.MMR_ROOT_ID(), data: abi.encodePacked(mmrRoot)
        });
        c.blockNumber = beefyBlock;
        c.validatorSetID = vsetId;
        c.payload = payload;
    }

    function _encodeCommitment(BeefyClient.Commitment memory commitment)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(
            _encodeCommitmentPayload(commitment.payload),
            ScaleCodec.encodeU32(commitment.blockNumber),
            ScaleCodec.encodeU64(commitment.validatorSetID)
        );
    }

    function _encodeCommitmentPayload(BeefyClient.PayloadItem[] memory items)
        internal
        pure
        returns (bytes memory)
    {
        bytes memory payload = ScaleCodec.checkedEncodeCompactU32(items.length);
        for (uint256 i = 0; i < items.length; i++) {
            payload = bytes.concat(
                payload,
                items[i].payloadID,
                ScaleCodec.checkedEncodeCompactU32(items[i].data.length),
                items[i].data
            );
        }
        return payload;
    }

    // merkle root + proofs (balanced power-of-two variant)
    function _merkleRootPow2(bytes32[] memory _leaves) internal pure returns (bytes32) {
        require(_leaves.length > 0, "no leaves");
        require(_isPow2(_leaves.length), "N must be power-of-two for this helper");
        bytes32[] memory level = _leaves;
        uint256 n = level.length;
        while (n > 1) {
            bytes32[] memory next = new bytes32[](n / 2);
            for (uint256 i = 0; i < n; i += 2) {
                next[i / 2] = keccak256(abi.encodePacked(level[i], level[i + 1]));
            }
            level = next;
            n = level.length;
        }
        return level[0];
    }

    function _merkleProofPow2(bytes32[] memory _leaves, uint256 index)
        internal
        pure
        returns (bytes32[] memory proof)
    {
        require(_leaves.length > 0, "no leaves");
        require(_isPow2(_leaves.length), "N must be power-of-two for this helper");
        require(index < _leaves.length, "index OOB");
        uint256 depth = _log2(_leaves.length);
        proof = new bytes32[](depth);
        bytes32[] memory level = _leaves;
        uint256 idx = index;
        uint256 n = level.length;
        for (uint256 d = 0; d < depth; d++) {
            uint256 sibling = idx ^ 1;
            proof[d] = level[sibling];
            // build next level
            bytes32[] memory next = new bytes32[](n / 2);
            for (uint256 i = 0; i < n; i += 2) {
                next[i / 2] = keccak256(abi.encodePacked(level[i], level[i + 1]));
            }
            level = next;
            n = level.length;
            idx >>= 1;
        }
    }

    function _isPow2(uint256 x) internal pure returns (bool) {
        return x != 0 && (x & (x - 1)) == 0;
    }

    function _log2(uint256 x) internal pure returns (uint256) {
        uint256 r = 0;
        while (x > 1) {
            x >>= 1;
            r++;
        }
        return r;
    }
}
