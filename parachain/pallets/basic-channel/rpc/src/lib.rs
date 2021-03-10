use codec::{Codec, Decode, Encode};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{H256};
use sp_runtime::{
	offchain::{
		storage::StorageValueRef,
	},
	traits::Block as BlockT,
};
use std::sync::Arc;

use artemis_basic_channel::outbound::{CommitmentData, Message, generate_merkle_proofs, offchain_key};

#[rpc]
pub trait BasicChannelApi
{
	#[rpc(name = "get_merkle_proofs")]
	fn get_merkle_proofs(&self, root: H256) -> Result<u64>;
}

pub struct BasicChannel<AccountId> {
	indexing_prefix: &'static [u8],
	_marker: std::marker::PhantomData<AccountId>
}

impl<AccountId> BasicChannel<AccountId> {
	pub fn new(indexing_prefix: &'static [u8]) -> Self {
		Self {
			indexing_prefix,
			_marker: Default::default(),
		}
	}
}

impl<AccountId> BasicChannelApi for BasicChannel<AccountId>
where
	AccountId: Codec + Send + Sync + 'static,
{
	fn get_merkle_proofs(&self, root: H256) -> Result<u64> {
		let key = offchain_key(self.indexing_prefix, root);
		let stored_data = StorageValueRef::persistent(b"offchain-demo::gh-info");

		if let Some(Some(raw_data)) = stored_data.get::<CommitmentData<AccountId>>() {
			return Ok(91919);
		}

		Ok(999999)
	}
}
