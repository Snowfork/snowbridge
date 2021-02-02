//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_core::H160;
use artemis_ethereum::Log;

pub mod types;
pub mod assets;

pub use types::{
	Message,
	Proof,
	ChannelId,
	SourceChannelConfig,
	SourceChannel,
};

pub use assets::{AssetId, MultiAsset, SingleAsset};
/// A trait for verifying messages.
///
/// This trait should be implemented by runtime modules that wish to provide message verification functionality.
pub trait Verifier {
	fn verify(message: &Message) -> Result<Log, DispatchError>;
}

/// Outbound submission for applications
pub trait SubmitOutbound {
	fn submit(channel_id: ChannelId, target: H160, payload: &[u8]) -> DispatchResult;
}

/// An Application handles message payloads
pub trait Application {

	/// Handle a message payload
	fn handle(payload: &[u8]) -> DispatchResult;

	fn address() -> H160;
}

/// Add a message to a commitment
pub trait MessageCommitment {
	fn add(channel_id: ChannelId, target: H160, nonce: u64, payload: &[u8]) -> DispatchResult;
}
