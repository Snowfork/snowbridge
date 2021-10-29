//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_system::Config;
use snowbridge_ethereum::{Header, Log, U256};
use sp_core::H160;
use sp_std::prelude::*;

pub mod assets;
pub mod nft;
pub mod types;

pub use types::{ChannelId, Message, MessageId, MessageNonce, Proof};

pub use assets::{AssetId, MultiAsset, SingleAsset};

pub use nft::{ERC721TokenData, TokenInfo};

/// A trait for verifying messages.
///
/// This trait should be implemented by runtime modules that wish to provide message verification
/// functionality.
pub trait Verifier {
	fn verify(message: &Message) -> Result<Log, DispatchError>;
	fn initialize_storage(
		headers: Vec<Header>,
		initial_difficulty: U256,
		descendants_until_final: u8,
	) -> Result<(), &'static str>;
}

/// Outbound submission for applications
pub trait OutboundRouter<AccountId> {
	fn submit(
		channel_id: ChannelId,
		who: &AccountId,
		target: H160,
		payload: &[u8],
	) -> DispatchResult;
}

/// Add a message to a commitment
pub trait MessageCommitment {
	fn add(channel_id: ChannelId, target: H160, nonce: u64, payload: &[u8]) -> DispatchResult;
}

/// Dispatch a message
pub trait MessageDispatch<T: Config, MessageId> {
	fn dispatch(source: H160, id: MessageId, payload: &[u8]);
	#[cfg(feature = "runtime-benchmarks")]
	fn successful_dispatch_event(id: MessageId) -> Option<<T as Config>::Event>;
}
