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

// Constants
const MMR_ROOT_ID: [u8; 2] = *b"mh";
const FIAT_SHAMIR_DOMAIN_ID: &[u8] = b"SNOWBRIDGE-FIAT-SHAMIR-v1";

// Global state variables (in a real implementation, these would be stored in the contract)
#[derive(Debug, Clone)]
struct State {
    latest_mmr_root: [u8; 32],
    latest_beefy_block: u64,
    current_validator_set: ValidatorSet,
    next_validator_set: ValidatorSet,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| {
    Mutex::new(State {
        latest_mmr_root: [0u8; 32],
        latest_beefy_block: 0,
        current_validator_set: ValidatorSet {
            id: 0,
            length: 0,
            root: [0u8; 32],
        },
        next_validator_set: ValidatorSet {
            id: 0,
            length: 0,
            root: [0u8; 32],
        },
    })
});

fn main() {
    submit_fiat_shamir();
}

fn submit_fiat_shamir() {
    // Read inputs
    let commitment = read::<Commitment>();
    let bitfield = read::<Vec<u64>>();
    let proofs = read::<Vec<ValidatorProof>>();
    let leaf = read::<MMRLeaf>();
    let leaf_proof = read::<Vec<[u8; 32]>>();
    let leaf_proof_order = read::<Vec<u8>>();

    // Verify all validator proofs
    let commitment_hash = hash_commitment(&commitment);
    let mut state = STATE.lock().unwrap();
    let validator_set = if (commitment.validator_set_id as u128) == state.current_validator_set.id {
        state.current_validator_set.clone()
    } else if (commitment.validator_set_id as u128) == state.next_validator_set.id {
        state.next_validator_set.clone()
    } else {
        panic!("Invalid commitment");
    };

    verify_fiat_shamir_commitment(&validator_set, &bitfield, &proofs, &commitment_hash);

    // Verify MMR proof
    verify_mmr_proof(&commitment, &leaf, &leaf_proof, &leaf_proof_order);

    // Update state
    state.latest_mmr_root = extract_mmr_root(&commitment);
    state.latest_beefy_block = commitment.block_number as u64;

    if (commitment.validator_set_id as u128) == state.next_validator_set.id {
        state.current_validator_set = state.next_validator_set.clone();
        state.next_validator_set = ValidatorSet {
            id: (commitment.validator_set_id as u128) + 1,
            length: leaf.next_authority_set_len as u128,
            root: leaf.next_authority_set_root,
        };
    }

    // Commit the results
    commit(&state.latest_mmr_root);
    commit(&state.latest_beefy_block.to_le_bytes());
}

// Helper functions

fn hash_commitment(commitment: &Commitment) -> [u8; 32] {
    let encoded = encode_commitment(commitment);
    keccak256(&encoded)
}

fn encode_commitment(commitment: &Commitment) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&encode_commitment_payload(&commitment.payload));
    out.extend_from_slice(&commitment.block_number.to_le_bytes());
    out.extend_from_slice(&commitment.validator_set_id.to_le_bytes());
    out
}

fn encode_commitment_payload(payload: &[PayloadItem]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&encode_compact_u32(payload.len() as u32));

    for item in payload {
        encoded.extend_from_slice(&item.payload_id);
        encoded.extend_from_slice(&encode_compact_u32(item.data.len() as u32));
        encoded.extend_from_slice(&item.data);
    }

    encoded
}

fn encode_compact_u32(value: u32) -> Vec<u8> {
    if value < 1 << 6 {
        vec![(value as u8) << 2]
    } else if value < 1 << 14 {
        let v = (value << 2) | 0x01;
        vec![(v & 0xff) as u8, ((v >> 8) & 0xff) as u8]
    } else if value < 1 << 30 {
        let v = (value << 2) | 0x02;
        vec![
            (v & 0xff) as u8,
            ((v >> 8) & 0xff) as u8,
            ((v >> 16) & 0xff) as u8,
            ((v >> 24) & 0xff) as u8,
        ]
    } else {
        panic!("CompactU32 too large");
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

fn is_bit_set(bitfield: &[u64], index: usize, length: usize) -> bool {
    if index >= length {
        return false;
    }

    let word_index = index / 64;
    let bit_index = index % 64;

    if word_index >= bitfield.len() {
        return false;
    }

    (bitfield[word_index] & (1u64 << bit_index)) != 0
}

fn count_set_bits(bitfield: &[u64], length: usize) -> usize {
    let mut count = 0;
    for i in 0..length {
        if is_bit_set(bitfield, i, length) {
            count += 1;
        }
    }
    count
}

fn hash_bitfield(bitfield: &[u64]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    for word in bitfield {
        hasher.update(&word.to_le_bytes());
    }
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

fn compute_max_required_signatures(length: usize) -> u32 {
    (length / 3 + 1) as u32
}

fn verify_validator_proofs(
    validator_set: &ValidatorSet,
    bitfield: &[u64],
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
    bitfield: &[u64],
    proofs: &[ValidatorProof],
    commitment_hash: &[u8; 32],
) {
    let required = compute_max_required_signatures(validator_set.length as usize) as usize;
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
    leaf_proof_order: &[u8],
) {
    let mmr_root = extract_mmr_root(commitment);
    let leaf_hash = keccak256(&encode_mmr_leaf(leaf));
    if leaf_proof.len() > 256 {
        panic!("Proof size exceeded");
    }

    let mut acc = leaf_hash;
    for (i, item) in leaf_proof.iter().enumerate() {
        let order = proof_order_bit(leaf_proof_order, i);
        acc = hash_pairs(acc, *item, order);
    }

    if acc != mmr_root {
        panic!("Invalid MMR leaf proof");
    }
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

fn fiat_shamir_final_bitfield(
    validator_set: &ValidatorSet,
    bitfield: &[u64],
    commitment_hash: &[u8; 32],
) -> Vec<u64> {
    let bitfield_hash = hash_bitfield(bitfield);
    let fiat_hash = create_fiat_shamir_hash(commitment_hash, &bitfield_hash, validator_set);
    let required = compute_max_required_signatures(validator_set.length as usize) as usize;
    subsample_bitfield(
        u64::from_be_bytes(fiat_hash[24..32].try_into().unwrap()),
        bitfield,
        validator_set.length as usize,
        required,
    )
}

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

fn encode_mmr_leaf(leaf: &MMRLeaf) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(leaf.version);
    out.extend_from_slice(&leaf.parent_number.to_le_bytes());
    out.extend_from_slice(&leaf.parent_hash);
    out.extend_from_slice(&leaf.next_authority_set_id.to_le_bytes());
    out.extend_from_slice(&leaf.next_authority_set_len.to_le_bytes());
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

fn subsample_bitfield(
    seed: u64,
    prior_bitfield: &[u64],
    prior_bitfield_size: usize,
    n: usize,
) -> Vec<u64> {
    if prior_bitfield.len() != container_length(prior_bitfield_size)
        || n > count_set_bits(prior_bitfield, prior_bitfield_size)
    {
        panic!("Invalid sampling params");
    }

    let mut output = vec![0u64; prior_bitfield.len()];
    let mut found = 0usize;
    let mut i = 0usize;

    while found < n {
        let index = make_index(seed, i as u64, prior_bitfield_size);
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

fn make_index(seed: u64, iteration: u64, length: usize) -> usize {
    if length == 0 {
        return 0;
    }
    let mut buf = [0u8; 16];
    buf[..8].copy_from_slice(&seed.to_le_bytes());
    buf[8..].copy_from_slice(&iteration.to_le_bytes());
    let hash = keccak256(&buf);
    let mut num = [0u8; 8];
    num.copy_from_slice(&hash[..8]);
    (u64::from_le_bytes(num) as usize) % length
}

fn container_length(bitfield_size: usize) -> usize {
    (bitfield_size + 63) / 64
}

fn set_bit(bitfield: &mut [u64], index: usize) {
    let word = index / 64;
    let bit = index % 64;
    bitfield[word] |= 1u64 << bit;
}

fn unset_bit(bitfield: &mut [u64], index: usize) {
    let word = index / 64;
    let bit = index % 64;
    bitfield[word] &= !(1u64 << bit);
}

fn proof_order_bit(order: &[u8], index: usize) -> u8 {
    if order.is_empty() {
        return 0;
    }
    let byte_index = index / 8;
    if byte_index >= order.len() {
        return 0;
    }
    (order[byte_index] >> (index % 8)) & 1
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
