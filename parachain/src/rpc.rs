#![warn(missing_docs)]

use std::sync::Arc;

use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;

#[cfg(feature = "with-snowbridge-runtime")]
use snowbridge_runtime::{opaque::Block, AccountId, COMMITMENTS_INDEXING_PREFIX};

#[cfg(feature = "with-rococo-runtime")]
use rococo_runtime::{opaque::Block, AccountId, COMMITMENTS_INDEXING_PREFIX};

#[cfg(feature = "with-local-runtime")]
use local_runtime::{opaque::Block, AccountId, COMMITMENTS_INDEXING_PREFIX};

use artemis_basic_channel_rpc::{BasicChannel, BasicChannelApi};

pub use jsonrpc_core;


/// Full client dependencies.
pub struct FullDeps<C, P, TBackend> {
	/// Backend, used for storage
	pub backend: Arc<TBackend>,
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, TBackend>(deps: FullDeps<C, P, TBackend>) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
	TBackend: sc_client_api::Backend<Block> + 'static,
{
	let mut io = jsonrpc_core::IoHandler::default();

	let FullDeps {
		backend,
		deny_unsafe,
		..
	} = deps;

	io.extend_with(BasicChannelApi::to_delegate(
		BasicChannel::<_, AccountId>::new(
			backend.offchain_storage().expect("requires backend with offchain storage"),
			deny_unsafe,
			COMMITMENTS_INDEXING_PREFIX)
	));

	io
}
