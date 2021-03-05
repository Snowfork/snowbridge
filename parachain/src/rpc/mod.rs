#![warn(missing_docs)]

use artemis_runtime::opaque::Block;
pub use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_transaction_pool::TransactionPool;
use std::sync::Arc;

pub use jsonrpc_core;

mod merkle_proofs_rpc;

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
	P: TransactionPool + 'static,
{
	let mut io = jsonrpc_core::IoHandler::default();
    // let FullDeps {
    //     client,
    //     pool,
    //     deny_unsafe,
    // } = deps;

	io.extend_with(merkle_proofs_rpc::SillyRpc::to_delegate(
		merkle_proofs_rpc::Silly{},
	));
	println!("*********************************** RPC EXTENDED");
    // io.extend_with(SystemApi::to_delegate(FullSystem::new(
    //     client.clone(),
    //     pool,
    //     deny_unsafe,
    // )));

	io
}
