//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;

use sp_std::prelude::*;
use sp_core::H160;

pub mod types;
pub mod assets;

pub use types::{
	AppId,
	Message,
	VerificationInput,
};

pub use assets::{AssetId, MultiAsset, SingleAsset};

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
	fn handle(payload: &[u8]) -> DispatchResult;

	fn address() -> H160;
}

pub trait Commitments {

	fn add(address: H160, payload: Vec<u8>);
}
