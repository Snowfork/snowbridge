#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;

pub mod types;
pub mod registry;

pub use types::{AppID, Message, VerificationInput, VerifiedMessage};

/// Verifier modules should implement this trait
pub trait Verifier<AccountId> {

	fn verify(sender: AccountId, app_id: AppID, message: &Message) -> DispatchResult;
}

/// Application modules should implement this trait
pub trait Application {

	/// Handle a message
	fn handle(message: Message) -> DispatchResult;
}
