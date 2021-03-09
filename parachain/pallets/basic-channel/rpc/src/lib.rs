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

use artemis_basic_channel::outbound::{CommitmentData, Message};
pub use artemis_basic_channel_runtime_api::BasicChannelApi as BasicChannelRuntimeApi;

#[rpc]
pub trait BasicChannelApi<AccountId>
{
	#[rpc(name = "get_merkle_proofs")]
	fn get_merkle_proofs(&self, root: H256) -> Result<u64>;
}

pub struct BasicChannel<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> BasicChannel<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block, AccountId> BasicChannelApi<AccountId> for BasicChannel<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: BasicChannelRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	fn get_merkle_proofs(&self, root: H256) -> Result<u64> {
		let stored_data = StorageValueRef::persistent(b"offchain-demo::gh-info");

		if let Some(Some(raw_data)) = stored_data.get::<CommitmentData<AccountId>>() {
			return Ok(91919);
		}

		Ok(999999)
	}
}
