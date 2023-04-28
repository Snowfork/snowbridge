use frame_support::log;
use sp_core::H256;
use sp_io::hashing::sha2_256;

pub fn is_valid_merkle_branch(
	leaf: H256,
	branch: &[H256],
	depth: u64,
	index: u64,
	root: H256,
) -> bool {
	if branch.len() as u64 != depth {
		log::error!("Merkle proof branch length doesn't match depth.");

		return false
	}
	let mut value = leaf;
	if leaf.as_bytes().len() < 32_usize {
		log::error!("Merkle proof leaf not 32 bytes.");

		return false
	}
	for i in 0..depth {
		if branch[i as usize].as_bytes().len() < 32 as usize {
			log::error!("Merkle proof branch not 32 bytes.");

			return false
		}
		if (index / (2u32.pow(i as u32) as u64) % 2) == 0 {
			// left node
			let mut data = [0u8; 64];
			data[0..32].copy_from_slice(&(value.0));
			data[32..64].copy_from_slice(&(branch[i as usize].0));
			value = sha2_256(&data).into();
		} else {
			let mut data = [0u8; 64]; // right node
			data[0..32].copy_from_slice(&(branch[i as usize].0));
			data[32..64].copy_from_slice(&(value.0));
			value = sha2_256(&data).into();
		}
	}

	value == root
}
