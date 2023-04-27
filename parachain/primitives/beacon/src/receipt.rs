use sp_core::H256;
use sp_io::hashing::keccak_256;
use sp_std::prelude::*;

use snowbridge_ethereum::{mpt, Receipt};

pub fn verify_receipt_proof(
	receipts_root: H256,
	proof: &[Vec<u8>],
) -> Option<Result<Receipt, rlp::DecoderError>> {
	match apply_merkle_proof(proof) {
		Some((root, data)) if root == receipts_root => Some(rlp::decode(&data)),
		Some((_, _)) => None,
		None => None,
	}
}

fn apply_merkle_proof(proof: &[Vec<u8>]) -> Option<(H256, Vec<u8>)> {
	let mut iter = proof.into_iter().rev();
	let first_bytes = match iter.next() {
		Some(b) => b,
		None => return None,
	};
	let item_to_prove: mpt::ShortNode = rlp::decode(first_bytes).ok()?;

	let final_hash: Option<[u8; 32]> =
		iter.fold(Some(keccak_256(first_bytes)), |maybe_hash, bytes| {
			let expected_hash = maybe_hash?;
			let node: Box<dyn mpt::Node> = bytes.as_slice().try_into().ok()?;
			if (*node).contains_hash(expected_hash.into()) {
				return Some(keccak_256(bytes))
			}
			None
		});

	final_hash.map(|hash| (hash.into(), item_to_prove.value))
}
