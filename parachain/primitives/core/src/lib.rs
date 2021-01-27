//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;

use sp_core::H160;

pub mod types;
pub mod assets;

pub use types::{
	AppId,
	Message,
	VerificationInput,
	ChannelId,
};

pub use assets::{AssetId, MultiAsset, SingleAsset};
/// A trait for verifying messages.
///
/// This trait should be implemented by runtime modules that wish to provide message verification functionality.
pub trait Verifier<AccountId> {

	fn verify(sender: AccountId, app_id: AppId, message: &Message) -> DispatchResult;

	fn verify_bulk(sender: AccountId, messages: &[(AppId, Message)]) -> DispatchResult;
}

impl<AccountId> Verifier<AccountId> for () {
	fn verify(sender: AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		Ok(())
	}
	fn verify_bulk(sender: AccountId, messages: &[(AppId, Message)]) -> DispatchResult {
		Ok(())
	}
}

/// Outbound submission for applications
pub trait SubmitOutbound {
	fn submit(channel_id: ChannelId, payload: &[u8]) -> DispatchResult;
}

impl SubmitOutbound for () {
	fn submit(channel_id: ChannelId, payload: &[u8]) -> DispatchResult {
		Ok(())
	}
}

/// An Application handles message payloads
pub trait Application {

	/// Handle a message payload
	fn handle(payload: &[u8]) -> DispatchResult;

	fn address() -> H160;
}

impl Application for () {
	fn handle(payload: &[u8]) -> DispatchResult {
		Ok(())
	}

	fn address() -> H160 {
		H160::zero()
	}
}

/// Add a message to a commitment
pub trait MessageCommitment {
	fn add(channel_id: ChannelId, address: H160, nonce: u64, payload: &[u8]);
}

impl MessageCommitment for () {
	fn add(channel_id: ChannelId, address: H160, nonce: u64, payload: &[u8]) { }
}
