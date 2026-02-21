use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sha3::{Digest, Keccak256};
use sp1_zkvm::io::{commit, read};
use std::sync::{LazyLock, Mutex};

// Types representing the core data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub block_number: u32,
    pub validator_set_id: u64,
    pub payload: Vec<PayloadItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadItem {
    pub payload_id: [u8; 2],
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorProof {
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub index: u32,
    pub account: [u8; 20],
    pub proof: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMRLeaf {
    pub version: u8,
    pub parent_number: u32,
    pub parent_hash: [u8; 32],
    pub next_authority_set_id: u64,
    pub next_authority_set_len: u32,
    pub next_authority_set_root: [u8; 32],
    pub parachain_heads_root: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub id: u128,
    pub length: u128,
    pub root: [u8; 32],
}

// Constants (must match BeefyClient.sol)
const MMR_ROOT_ID: [u8; 2] = *b"mh";
const FIAT_SHAMIR_DOMAIN_ID: &[u8] = b"SNOWBRIDGE-FIAT-SHAMIR-v1";

/// Bitfield element: 32 bytes (uint256) matching Solidity Bitfield.sol
type BitfieldElement = [u8; 32];
const FIAT_SHAMIR_REQUIRED_SIGNATURES: u32 = 111;

/// State: on-chain values. Validator sets are fixed at deployment (initialization).
/// latest_mmr_root and latest_beefy_block are the prior verified values.
#[derive(Debug, Clone)]
struct State {
    latest_mmr_root: [u8; 32],
    latest_beefy_block: u64,
    current_validator_set: ValidatorSet,
    next_validator_set: ValidatorSet,
}

// Initial state baked in at build time from contracts/test/data/checkpoint.json
include!(concat!(env!("OUT_DIR"), "/checkpoint.rs"));

/// Current state, initialized from checkpoint and updated on each successful submit_fiat_shamir.
static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(INITIAL_STATE.clone()));

fn main() {
    submit_fiat_shamir();
}

fn submit_fiat_shamir() {
    // Read inputs (formats must match BeefyClient.sol)
    let commitment = read::<Commitment>();
    let bitfield = read::<Vec<BitfieldElement>>();
    let proofs = read::<Vec<ValidatorProof>>();
    let leaf = read::<MMRLeaf>();
    let leaf_proof = read::<Vec<[u8; 32]>>();
    let leaf_proof_order = read::<[u8; 32]>(); // uint256, bits 0..=n for proof order

    // Verify all validator proofs
    let commitment_hash = hash_commitment(&commitment);

    let mut state = STATE.lock().unwrap();

    // Check for stale commitment
    if commitment.block_number as u64 <= state.latest_beefy_block {
        panic!("Stale commitment");
    }

    let validator_set = if (commitment.validator_set_id as u128) == state.current_validator_set.id {
        state.current_validator_set.clone()
    } else if (commitment.validator_set_id as u128) == state.next_validator_set.id {
        state.next_validator_set.clone()
    } else {
        panic!("Invalid commitment");
    };

    // Validate bitfield with quorum check (2/3 + 1)
    let quorum = compute_quorum(validator_set.length as usize);
    validate_bitfield(&bitfield, validator_set.length as usize, quorum);

    verify_fiat_shamir_commitment(&validator_set, &bitfield, &proofs, &commitment_hash);

    let is_next_session = (commitment.validator_set_id as u128) == state.next_validator_set.id;

    // Verify MMR proof only when transitioning to next validator set (matches BeefyClient.sol)
    if is_next_session {
        if leaf.next_authority_set_id != (state.next_validator_set.id + 1) as u64 {
            panic!("Invalid MMR leaf");
        }
        verify_mmr_proof(&commitment, &leaf, &leaf_proof, &leaf_proof_order);
    }

    // Update STATE on success
    state.latest_mmr_root = extract_mmr_root(&commitment);
    state.latest_beefy_block = commitment.block_number as u64;

    if is_next_session {
        state.current_validator_set = state.next_validator_set.clone();
        state.next_validator_set = ValidatorSet {
            id: (commitment.validator_set_id as u128) + 1,
            length: leaf.next_authority_set_len as u128,
            root: leaf.next_authority_set_root,
        };
    }
    // STATE is held in Mutex and persists for the duration of this execution

    // Commit the results (order must match SP1BeefyClient.sol)
    commit(&state.latest_mmr_root);
    commit(&state.latest_beefy_block.to_le_bytes());
    commit(&commitment_hash);
}

// Helper functions

fn hash_commitment(commitment: &Commitment) -> [u8; 32] {
    let encoded = encode_commitment(commitment);
    keccak256(&encoded)
}

/// ScaleCodec.encodeCommitment - must match BeefyClient.sol
fn encode_commitment(commitment: &Commitment) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&encode_commitment_payload(&commitment.payload));
    out.extend_from_slice(&scale_encode_u32(commitment.block_number));
    out.extend_from_slice(&scale_encode_u64(commitment.validator_set_id));
    out
}

fn encode_commitment_payload(payload: &[PayloadItem]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&scale_encode_compact_u32(payload.len() as u32));

    for item in payload {
        encoded.extend_from_slice(&item.payload_id);
        encoded.extend_from_slice(&scale_encode_compact_u32(item.data.len() as u32));
        encoded.extend_from_slice(&item.data);
    }

    encoded
}

/// ScaleCodec.encodeU32 - byte-reversed (big-endian output)
fn scale_encode_u32(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

/// ScaleCodec.encodeU64 - byte-reversed (big-endian output)
fn scale_encode_u64(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

/// ScaleCodec.encodeCompactU32 - reverse16/reverse32 swap bytes
fn scale_encode_compact_u32(value: u32) -> Vec<u8> {
    if value <= 63 {
        vec![(value << 2) as u8]
    } else if value <= (1 << 14) - 1 {
        let v = ((value << 2) | 1) as u16;
        let b = v.to_be_bytes();
        vec![b[1], b[0]]
    } else if value <= (1 << 30) - 1 {
        let v = (value << 2) | 2;
        let b = v.to_be_bytes();
        vec![b[3], b[2], b[1], b[0]]
    } else {
        let mut out = vec![3u8];
        let b = value.to_be_bytes();
        out.extend_from_slice(&[b[3], b[2], b[1], b[0]]);
        out
    }
}

fn verify_signature(
    account: &[u8; 20],
    v: &u8,
    r: &[u8; 32],
    s: &[u8; 32],
    commitment_hash: &[u8; 32],
) -> bool {
    let mut v_normalized = *v;
    if v_normalized >= 27 {
        v_normalized -= 27;
    }

    let Some(recovery_id) = RecoveryId::from_byte(v_normalized) else {
        return false;
    };
    let Ok(signature) = Signature::from_scalars(*r, *s) else {
        return false;
    };
    let Ok(verify_key) =
        VerifyingKey::recover_from_prehash(commitment_hash, &signature, recovery_id)
    else {
        return false;
    };

    let pubkey = verify_key.to_encoded_point(false);
    let pubkey_bytes = pubkey.as_bytes();
    if pubkey_bytes.len() != 65 {
        return false;
    }

    let hash = Keccak256::digest(&pubkey_bytes[1..]);
    let mut derived = [0u8; 20];
    derived.copy_from_slice(&hash[12..]);
    &derived == account
}

fn verify_validator_in_set(
    validator_set: &ValidatorSet,
    account: &[u8; 20],
    index: u32,
    proof: &[[u8; 32]],
) -> bool {
    let leaf = keccak256(account);
    substrate_merkle_verify(
        &validator_set.root,
        &leaf,
        index as usize,
        validator_set.length as usize,
        proof,
    )
}

/// Bitfield.isSet - element = index >> 8, bit = index % 256 (matches Bitfield.sol)
fn is_bit_set(bitfield: &[BitfieldElement], index: usize, length: usize) -> bool {
    if index >= length {
        return false;
    }

    let element_index = index >> 8;
    if element_index >= bitfield.len() {
        return false;
    }

    let element = &bitfield[element_index];
    let byte_index = 31 - (index / 8);
    let bit_index = index % 8;
    ((element[byte_index] >> bit_index) & 1) != 0
}

fn count_set_bits(bitfield: &[BitfieldElement], length: usize) -> usize {
    let mut count = 0;
    for i in 0..length {
        if is_bit_set(bitfield, i, length) {
            count += 1;
        }
    }
    count
}

/// keccak256(abi.encodePacked(bitfield)) - matches BeefyClient fiatShamirFinalBitfield
fn hash_bitfield(bitfield: &[BitfieldElement]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    for element in bitfield {
        hasher.update(element);
    }
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// 2/3 + 1 of validators (quorum) - bitfield must have at least this many set bits.
fn compute_quorum(length: usize) -> u32 {
    (length - (length - 1) / 3) as u32
}

/// 1/3 + 1 - sufficient to ensure at least one honest validator when verifying signatures.
fn compute_max_required_signatures(length: usize) -> u32 {
    (length / 3 + 1) as u32
}

fn verify_validator_proofs(
    validator_set: &ValidatorSet,
    bitfield: &[BitfieldElement],
    proofs: &[ValidatorProof],
    commitment_hash: &[u8; 32],
) {
    let mut available = bitfield.to_vec();
    for proof in proofs {
        if !is_bit_set(
            &available,
            proof.index as usize,
            validator_set.length as usize,
        ) {
            panic!("Validator not in bitfield");
        }

        if !verify_validator_in_set(validator_set, &proof.account, proof.index, &proof.proof) {
            panic!("Invalid validator proof");
        }

        if !verify_signature(
            &proof.account,
            &proof.v,
            &proof.r,
            &proof.s,
            commitment_hash,
        ) {
            panic!("Invalid signature");
        }

        unset_bit(&mut available, proof.index as usize);
    }
}

fn verify_fiat_shamir_commitment(
    validator_set: &ValidatorSet,
    bitfield: &[BitfieldElement],
    proofs: &[ValidatorProof],
    commitment_hash: &[u8; 32],
) {
    let max_required = compute_max_required_signatures(validator_set.length as usize) as usize;
    let required = std::cmp::min(FIAT_SHAMIR_REQUIRED_SIGNATURES as usize, max_required);
    if proofs.len() != required {
        panic!("Invalid validator proof length");
    }
    let final_bitfield = fiat_shamir_final_bitfield(validator_set, bitfield, commitment_hash);
    verify_validator_proofs(validator_set, &final_bitfield, proofs, commitment_hash);
}

fn verify_mmr_proof(
    commitment: &Commitment,
    leaf: &MMRLeaf,
    leaf_proof: &[[u8; 32]],
    leaf_proof_order: &[u8; 32],
) {
    let mmr_root = extract_mmr_root(commitment);
    let leaf_hash = keccak256(&encode_mmr_leaf(leaf));

    // Check proof size
    if leaf_proof.len() > 256 {
        panic!("Proof size exceeded");
    }

    // Verify MMR leaf proof - matching Solidity MMRProof.verifyLeafProof
    let leaf_is_valid = mmr_verify_leaf_proof(&mmr_root, &leaf_hash, leaf_proof, leaf_proof_order);

    if !leaf_is_valid {
        panic!("Invalid MMR leaf proof");
    }
}

/// MMRProof.verifyLeafProof - (proofOrder >> i) & 1 for order
fn mmr_verify_leaf_proof(
    root: &[u8; 32],
    leaf: &[u8; 32],
    leaf_proof: &[[u8; 32]],
    leaf_proof_order: &[u8; 32],
) -> bool {
    let mut acc = *leaf;
    for (i, item) in leaf_proof.iter().enumerate() {
        let order = proof_order_bit(leaf_proof_order, i);
        acc = hash_pairs(acc, *item, order);
    }
    acc == *root
}

fn extract_mmr_root(commitment: &Commitment) -> [u8; 32] {
    // Find the MMR root payload item
    if commitment.payload.len() != 1 {
        panic!("Commitment not relevant");
    }

    for item in &commitment.payload {
        if item.payload_id == MMR_ROOT_ID {
            if item.data.len() != 32 {
                panic!("Invalid MMR root length");
            }
            let mut root = [0u8; 32];
            root.copy_from_slice(&item.data);
            return root;
        }
    }
    panic!("MMR root not found in commitment");
}

/// fiatShamirFinalBitfield - seed = uint256(fiatShamirHash) (full 32 bytes)
fn fiat_shamir_final_bitfield(
    validator_set: &ValidatorSet,
    bitfield: &[BitfieldElement],
    commitment_hash: &[u8; 32],
) -> Vec<BitfieldElement> {
    let bitfield_hash = hash_bitfield(bitfield);
    let fiat_hash = create_fiat_shamir_hash(commitment_hash, &bitfield_hash, validator_set);

    let max_required = compute_max_required_signatures(validator_set.length as usize) as usize;
    let required = std::cmp::min(FIAT_SHAMIR_REQUIRED_SIGNATURES as usize, max_required);
    subsample_bitfield(
        fiat_hash,
        bitfield,
        validator_set.length as usize,
        required,
    )
}

/// createFiatShamirHash - bytes32(uint256(v)) for id/length (must match BeefyClient.sol)
fn create_fiat_shamir_hash(
    commitment_hash: &[u8; 32],
    bitfield_hash: &[u8; 32],
    validator_set: &ValidatorSet,
) -> [u8; 32] {
    let mut inner = Sha256::new();
    inner.update(commitment_hash);
    inner.update(bitfield_hash);
    inner.update(&validator_set.root);
    inner.update(&u128_to_be_bytes_32(validator_set.id));
    inner.update(&u128_to_be_bytes_32(validator_set.length));
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(FIAT_SHAMIR_DOMAIN_ID);
    outer.update(inner_hash);
    let result = outer.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// encodeMMRLeaf - ScaleCodec encoding (must match BeefyClient.sol)
fn encode_mmr_leaf(leaf: &MMRLeaf) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(leaf.version);
    out.extend_from_slice(&scale_encode_u32(leaf.parent_number));
    out.extend_from_slice(&leaf.parent_hash);
    out.extend_from_slice(&scale_encode_u64(leaf.next_authority_set_id));
    out.extend_from_slice(&scale_encode_u32(leaf.next_authority_set_len));
    out.extend_from_slice(&leaf.next_authority_set_root);
    out.extend_from_slice(&leaf.parachain_heads_root);
    out
}

fn substrate_merkle_verify(
    root: &[u8; 32],
    leaf: &[u8; 32],
    position: usize,
    width: usize,
    proof: &[[u8; 32]],
) -> bool {
    if position >= width {
        return false;
    }
    let computed = substrate_merkle_root(*leaf, position, width, proof);
    &computed == root
}

fn substrate_merkle_root(
    leaf: [u8; 32],
    mut position: usize,
    mut width: usize,
    proof: &[[u8; 32]],
) -> [u8; 32] {
    let mut node = leaf;
    for item in proof {
        node = if (position & 1 == 1) || (position + 1 == width) {
            keccak256_pairs(item, &node)
        } else {
            keccak256_pairs(&node, item)
        };
        position >>= 1;
        width = ((width - 1) >> 1) + 1;
    }
    node
}

/// Bitfield.subsample - keccak256(abi.encodePacked(seed, iteration)), seed/iteration uint256
fn subsample_bitfield(
    seed: [u8; 32],
    prior_bitfield: &[BitfieldElement],
    prior_bitfield_size: usize,
    n: usize,
) -> Vec<BitfieldElement> {
    if prior_bitfield.len() != container_length(prior_bitfield_size)
        || n > count_set_bits(prior_bitfield, prior_bitfield_size)
    {
        panic!("Invalid sampling params");
    }

    let mut output = vec![[0u8; 32]; prior_bitfield.len()];
    let mut found = 0usize;
    let mut i = 0u64;

    while found < n {
        let index = make_index(seed, i, prior_bitfield_size);
        if !is_bit_set(prior_bitfield, index, prior_bitfield_size)
            || is_bit_set(&output, index, prior_bitfield_size)
        {
            i += 1;
            continue;
        }

        set_bit(&mut output, index);
        found += 1;
        i += 1;
    }

    output
}

/// Bitfield.makeIndex - keccak256(abi.encodePacked(seed, iteration)) % length
/// Both seed and iteration are uint256 (32 bytes); iteration is zero-padded for u64
fn make_index(seed: [u8; 32], iteration: u64, length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&seed);
    let mut iter_padded = [0u8; 32];
    iter_padded[24..].copy_from_slice(&iteration.to_be_bytes());
    buf[32..].copy_from_slice(&iter_padded);
    let hash = keccak256(&buf);
    u256_mod_usize(&hash, length)
}

/// Compute bytes32 as uint256 mod n (matches Solidity mod(keccak256(...), length))
fn u256_mod_usize(bytes: &[u8; 32], n: usize) -> usize {
    let mut result: u128 = 0;
    for &b in bytes {
        result = (result * 256 + b as u128) % n as u128;
    }
    result as usize
}

fn container_length(bitfield_size: usize) -> usize {
    (bitfield_size + 255) / 256
}

fn validate_bitfield_padding(bitfield: &[BitfieldElement], length: usize) {
    let container_len = container_length(length);
    if container_len == 0 || bitfield.is_empty() {
        return;
    }

    let valid_bits_in_last_element = length % 256;
    if valid_bits_in_last_element == 0 {
        return;
    }

    let last_element = &bitfield[container_len - 1];
    for bit in valid_bits_in_last_element..256 {
        let byte_index = 31 - (bit / 8);
        let bit_index = bit % 8;
        if (last_element[byte_index] >> bit_index) & 1 != 0 {
            panic!("Invalid bitfield padding");
        }
    }
}

fn validate_bitfield(bitfield: &[BitfieldElement], length: usize, required_min_bits: u32) {
    // Check bitfield length matches container length
    let expected_container_len = container_length(length);
    if bitfield.len() != expected_container_len {
        panic!("Invalid bitfield length");
    }

    // Check that bitfield has minimum set bits (quorum)
    let set_bits = count_set_bits(bitfield, length);
    if set_bits < required_min_bits as usize {
        panic!("Insufficient set bits in bitfield");
    }

    // Validate padding bits are zero
    validate_bitfield_padding(bitfield, length);
}

fn set_bit(bitfield: &mut [BitfieldElement], index: usize) {
    let element_index = index >> 8;
    let byte_index = 31 - (index / 8);
    let bit_index = index % 8;
    bitfield[element_index][byte_index] |= 1 << bit_index;
}

fn unset_bit(bitfield: &mut [BitfieldElement], index: usize) {
    let element_index = index >> 8;
    let byte_index = 31 - (index / 8);
    let bit_index = index % 8;
    bitfield[element_index][byte_index] &= !(1 << bit_index);
}

/// Extract bit i from uint256 proofOrder: (proofOrder >> i) & 1
fn proof_order_bit(order: &[u8; 32], index: usize) -> u8 {
    if index >= 256 {
        return 0;
    }
    let byte_index = 31 - (index / 8);
    ((order[byte_index] >> (index % 8)) & 1) as u8
}

fn hash_pairs(x: [u8; 32], y: [u8; 32], order: u8) -> [u8; 32] {
    if order == 0 {
        keccak256_pairs(&x, &y)
    } else {
        keccak256_pairs(&y, &x)
    }
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

fn keccak256_pairs(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(a);
    hasher.update(b);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

fn u128_to_be_bytes_32(value: u128) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[16..].copy_from_slice(&value.to_be_bytes());
    out
}
