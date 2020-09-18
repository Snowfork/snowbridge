//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;

use sp_std::vec::Vec;

pub mod types;
pub mod registry;

pub use types::{
	AppId,
	Message,
	VerificationInput,
	BridgedAssetId
};

/// A trait for verifying messages.
///
/// This trait should be implemented by runtime modules that wish to provide message verification functionality.
pub trait Verifier<AccountId> {

	fn verify(sender: AccountId, app_id: AppId, message: &Message) -> DispatchResult;
}

/// A trait for handling message payloads.
///
/// This trait should be implemented by runtime modules that wish to handle message payloads.
pub trait Application {

	/// Handle a message payload
	fn handle(payload: Vec<u8>) -> DispatchResult;
}
