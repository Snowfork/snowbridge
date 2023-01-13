//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchError;
use frame_system::Config;
use snowbridge_ethereum::{Header, Log, U256};
use sp_core::H160;
use sp_std::prelude::*;

pub mod types;

pub use types::{Message, MessageId, MessageNonce, Proof};

/// A trait for verifying messages.
///
/// This trait should be implemented by runtime modules that wish to provide message verification
/// functionality.
pub trait Verifier {
	fn verify(message: &Message) -> Result<(Log, u64), DispatchError>;
	fn initialize_storage(
		headers: Vec<Header>,
		initial_difficulty: U256,
		descendants_until_final: u8,
	) -> Result<(), &'static str>;
}

/// Dispatch a message
pub trait MessageDispatch<T: Config, MessageId> {
	fn dispatch(source: H160, id: MessageId, payload: &[u8]);
	#[cfg(feature = "runtime-benchmarks")]
	fn successful_dispatch_event(id: MessageId) -> Option<<T as Config>::RuntimeEvent>;
}
