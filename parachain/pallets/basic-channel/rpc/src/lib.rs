use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{traits::Block as BlockT};
use std::sync::Arc;

pub use artemis_basic_channel_runtime_api::BasicChannelApi as BasicChannelRuntimeApi;

#[rpc]
pub trait BasicChannelApi {
	#[rpc(name = "get_merkle_proofs")]
	fn get_merkle_proofs(&self) -> Result<u64>;
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

impl<C, Block> BasicChannelApi for BasicChannel<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: BasicChannelRuntimeApi<Block>,
{
	fn get_merkle_proofs(&self) -> Result<u64> {
		Ok(999999)
	}
}
