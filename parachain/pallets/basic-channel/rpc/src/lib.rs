use std::sync::Arc;

use jsonrpc_core::{Result, Error, ErrorCode};
use jsonrpc_derive::rpc;

use codec::{Decode, Encode};
use sp_core::H256;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::generic::BlockId;
use sp_runtime::offchain::storage::StorageValueRef;

pub use snowbridge_basic_channel_rpc_runtime_api::BasicOutboundChannelApi as BasicOutboundChannelRuntimeApi;

#[derive(Encode, Decode)]
pub struct StoredLeaves(pub Vec<Vec<u8>>);

#[rpc]
pub trait BasicChannelApi<BlockHash> {
	#[rpc(name = "basicChannel_helloWorld")]
	fn hello_world(&self, at: Option<BlockHash>) -> Result<String>;

	#[rpc(name = "basicChannel_getMerkleProof")]
	fn get_merkle_proof(&self, at: Option<BlockHash>, commitment_hash: H256, leaf_index: u64) -> Result<Vec<u8>>;
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
	C::Api: BasicOutboundChannelRuntimeApi<Block>
{
	fn hello_world(&self, _at: Option<<Block as BlockT>::Hash>) ->  Result<String> {
		let answer = 42;

		Ok(format!("hello world! The answer is {}", answer).to_string())
	}

	fn get_merkle_proof(&self, at: Option<<Block as BlockT>::Hash>, commitment_hash: H256, leaf_index: u64) -> Result<Vec<u8>> {
		let oci_mem = StorageValueRef::persistent(&commitment_hash.as_bytes());

		if let Ok(Some(StoredLeaves(leaves))) = oci_mem.get::<StoredLeaves>() {
			let api = self.client.runtime_api();
			let block_hash = at.unwrap_or_else(||
				// If the block hash is not supplied assume the best block.
				self.client.info().best_hash);

			// TODO: why the *_with_context call here?
			let proof = api.generate_proof_with_context(
				&BlockId::hash(block_hash),
				sp_core::ExecutionContext::OffchainCall(None),
				leaves,
				leaf_index
			).unwrap().unwrap();
			// TODO: handle these unwraps

			Ok(proof)
		} else {
			// failed to retrieve leaves for commitment_hash
			Err(Error {
				code: ErrorCode::InternalError,
				message: format!("Failed to retrieve leaves for commitment_hash {}", commitment_hash),
				data: None
			})
		}
	}
}
