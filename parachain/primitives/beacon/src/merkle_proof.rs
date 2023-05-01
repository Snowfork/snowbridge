use sp_core::H256;
use sp_io::hashing::sha2_256;

// Reference https://github.com/ethereum/consensus-specs/blob/dev/ssz/merkle-proofs.md
// p.s. index here is actually [subtree_index](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#get_subtree_index)
pub fn verify_merkle_proof(leaf: H256, branch: &[H256], index: u64, root: H256) -> bool {
	let mut value: [u8; 32] = leaf.into();
	for (i, node) in branch.iter().enumerate() {
		let mut data = [0u8; 64];
		if (index / (2u32.pow(i as u32) as u64) % 2) == 0 {
			// left node
			data[0..32].copy_from_slice(&value);
			data[32..64].copy_from_slice(&node.0);
			value = sha2_256(&data);
		} else {
			// right node
			data[0..32].copy_from_slice(&node.0);
			data[32..64].copy_from_slice(&value);
			value = sha2_256(&data);
		}
	}
	value == root.0
}
