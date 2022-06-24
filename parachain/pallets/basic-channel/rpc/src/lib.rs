use std::sync::Arc;

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::generic::BlockId;

pub use basic_channel::outbound::BasicChannelOutboundRuntimeApi as OutboundRuntimeApi;

#[rpc]
pub trait BasicChannelApi<BlockHash> {
	#[rpc(name = "basicChannel_helloWorld")]
	fn hello_world(&self, at: Option<BlockHash>) -> Result<String>;

	// #[rpc(name = "basicChannel_getMerkleProof")]
	// fn get_merkle_proof(&self, leaf_index: u64) -> Result<Vec<u8>> {
	// 	Ok(Vec::new())
	// }
}

pub struct BasicChannel<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>
}

impl<C, B> BasicChannel<C, B> {
	pub fn new(client: Arc<C>) ->  Self {
	    Self { client, _marker: Default::default() }
	}
}

impl<C, Block> BasicChannelApi<<Block as BlockT>::Hash> for BasicChannel<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: OutboundRuntimeApi<Block>
{
	fn hello_world(&self, at: Option<<Block as BlockT>::Hash>) ->  Result<String> {
		let api = self.client.runtime_api();
		let block_hash = at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash);

		let answer = api.generate_proof(&BlockId::hash(block_hash), 3).unwrap().unwrap();

		Ok(format!("hello world! The answer is {}", answer).to_string())
	}
}
