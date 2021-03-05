use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{traits::Block as BlockT};
use std::sync::Arc;

pub use artemis_rialto_channel_runtime_api::RialtoChannelApi as RialtoChannelRuntimeApi;

#[rpc]
pub trait RialtoChannelApi {
	#[rpc(name = "get_merkle_roots")]
	fn get_merkle_roots(&self) -> Result<u64>;
}

pub struct RialtoChannel<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> RialtoChannel<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block> RialtoChannelApi for RialtoChannel<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: RialtoChannelRuntimeApi<Block>,
{
	fn get_merkle_roots(&self) -> Result<u64> {
		Ok(999999)
	}
}
