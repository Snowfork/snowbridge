use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use codec::{Decode, Encode};
use parking_lot::RwLock;
use sp_core::{offchain::OffchainStorage, Bytes, H256};
use sp_runtime::traits::Keccak256;

use std::sync::Arc;

use snowbridge_basic_channel_merkle_proof::merkle_proof;

pub struct BasicChannel<T: OffchainStorage> {
	storage: Arc<RwLock<T>>,
}

impl<T: OffchainStorage> BasicChannel<T> {
	pub fn new(storage: T) -> Self {
		Self { storage: Arc::new(RwLock::new(storage)) }
	}
}

#[derive(Decode)]
struct Leaves(pub Vec<Vec<u8>>);

#[rpc]
pub trait BasicChannelApi {
	#[rpc(name = "basicOutboundChannel_getMerkleProof")]
	fn get_merkle_proof(&self, commitment_hash: H256, leaf_index: u64) -> Result<Bytes>;
}

impl<T> BasicChannelApi for BasicChannel<T>
where
	T: OffchainStorage + 'static,
{
	fn get_merkle_proof(&self, commitment_hash: H256, leaf_index: u64) -> Result<Bytes> {
		let encoded_leaves = match self
			.storage
			.read()
			.get(sp_offchain::STORAGE_PREFIX, commitment_hash.as_bytes())
		{
			Some(encoded_leaves) => encoded_leaves,
			None => {
				return Err(Error {
					code: ErrorCode::InvalidParams,
					message: "no leaves found for given commitment".into(),
					data: None,
				})
			},
		};

		let leaves = match Leaves::decode(&mut encoded_leaves.as_ref()) {
			Ok(leaves) => leaves,
			Err(_) => {
				return Err(Error {
					code: ErrorCode::InternalError,
					message: "could not decode leaves from storage".into(),
					data: None,
				})
			},
		};

		if (leaf_index as usize) >= Vec::len(&leaves.0) {
			return Err(Error {
				code: ErrorCode::InvalidParams,
				message: "leaf_index out of range".into(),
				data: None,
			});
		}

		let proof = merkle_proof::<Keccak256, Vec<Vec<u8>>, Vec<u8>>(leaves.0, leaf_index);
		Ok(proof.encode().into())
	}
}

#[cfg(test)]
mod tests {
	use crate::{BasicChannel, BasicChannelApi};
	use codec::Encode;
	use jsonrpc_core::{Error, ErrorCode};
	use sp_core::offchain::OffchainStorage;

	#[derive(Clone)]
	struct MockOffchainStorage<'a> {
		prefix: &'a [u8],
		key: &'a [u8],
		value: Option<Vec<u8>>,
	}
	impl<'a> OffchainStorage for MockOffchainStorage<'a> {
		fn set(&mut self, _prefix: &[u8], _key: &[u8], _value: &[u8]) {}
		fn remove(&mut self, _prefix: &[u8], _key: &[u8]) {}
		fn compare_and_set(
			&mut self,
			_prefix: &[u8],
			_key: &[u8],
			_old_value: Option<&[u8]>,
			_new_value: &[u8],
		) -> bool {
			false
		}

		fn get(&self, prefix: &[u8], key: &[u8]) -> Option<Vec<u8>> {
			if prefix == self.prefix && key == self.key {
				self.value.clone()
			} else {
				None
			}
		}
	}

	const TEST_HASH: &[u8; 32] = &[0; 32];
	fn create_rpc_handler<'a>(
		prefix: &'a [u8],
		key: &'a [u8],
		value: Option<Vec<u8>>,
	) -> BasicChannel<MockOffchainStorage<'a>> {
		let storage = MockOffchainStorage { prefix, key, value };
		BasicChannel::new(storage)
	}

	#[test]
	fn basic_channel_rpc_should_create_proof_for_existing_commitment() {
		let encoded_leaves = hex::decode("088107000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000006000000000000000000000000b8ea8cb425d85536b158d661da1ef0895bb92f1d000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000647ed9db598eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001bc16d674ec8000000000000000000000000000000000000000000000000000000000000810700000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000005000000000000000000000000b8ea8cb425d85536b158d661da1ef0895bb92f1d000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000647ed9db59d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001bc16d674ec8000000000000000000000000000000000000000000000000000000000000")
			.expect("test input should decode successfully");
		let rpc_handler =
			create_rpc_handler(sp_offchain::STORAGE_PREFIX, TEST_HASH, Some(encoded_leaves));

		let result = rpc_handler
			.get_merkle_proof(TEST_HASH.into(), 0)
			.expect("test input should have a Merkle proof")
			.to_vec();
		let expected_proof = hex::decode("1145ecaf4f9ee757a1bbcd41ae26b43a75c0a16e07c01d3502af4a480c28cbb30485ab07a8698e29740bbbad18710faa8f055e9d398efd80ffd7ea6f76348aa803020000000000000000000000000000008107000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000006000000000000000000000000b8ea8cb425d85536b158d661da1ef0895bb92f1d000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000647ed9db598eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001bc16d674ec8000000000000000000000000000000000000000000000000000000000000")
			.expect("test proof should decode successfully");

		assert_eq!(result, expected_proof);
	}

	#[test]
	fn basic_channel_rpc_should_handle_non_existent_commitment() {
		let rpc_handler = create_rpc_handler(sp_offchain::STORAGE_PREFIX, TEST_HASH, None);

		let result = rpc_handler.get_merkle_proof(TEST_HASH.into(), 0);

		assert_eq!(
			result,
			Err(Error {
				code: ErrorCode::InvalidParams,
				message: "no leaves found for given commitment".into(),
				data: None,
			})
		)
	}

	#[test]
	fn basic_channel_rpc_should_handle_incorrectly_encoded_leaves() {
		let rpc_handler =
			create_rpc_handler(sp_offchain::STORAGE_PREFIX, TEST_HASH, Some([42].to_vec()));

		let result = rpc_handler.get_merkle_proof(TEST_HASH.into(), 0);

		assert_eq!(
			result,
			Err(Error {
				code: ErrorCode::InternalError,
				message: "could not decode leaves from storage".into(),
				data: None,
			})
		)
	}

	#[test]
	fn basic_channel_rpc_should_handle_leaf_index_out_of_bounds() {
		let leaves: Vec<Vec<u8>> = vec![vec![1, 2], vec![3, 4]];
		let rpc_handler =
			create_rpc_handler(sp_offchain::STORAGE_PREFIX, TEST_HASH, Some(leaves.encode()));

		let result = rpc_handler.get_merkle_proof(TEST_HASH.into(), 2);

		assert_eq!(
			result,
			Err(Error {
				code: ErrorCode::InvalidParams,
				message: "leaf_index out of range".into(),
				data: None,
			})
		)
	}
}
