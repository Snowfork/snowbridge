#![warn(missing_docs)]

use std::sync::Arc;

use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;

use artemis_runtime::opaque::Block;
use artemis_basic_channel_rpc::{BasicChannel, BasicChannelApi, BasicChannelRuntimeApi};

pub use jsonrpc_core;


/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(deps: FullDeps<C, P>) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: BlockBuilder<Block>,
	C::Api: BasicChannelRuntimeApi<Block>,
	P: TransactionPool + 'static,
{
	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		..
	} = deps;

	io.extend_with(BasicChannelApi::to_delegate(
		BasicChannel::new(client),
	));

	io
}
