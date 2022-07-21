use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use codec::{Decode, Encode};
use parking_lot::RwLock;
use sp_core::{offchain::OffchainStorage, H256, Bytes};
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

		let proof = merkle_proof::<Keccak256, Vec<Vec<u8>>, Vec<u8>>(leaves.0, leaf_index);
		Ok(proof.encode().into())
	}
}
