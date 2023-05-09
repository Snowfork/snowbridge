//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchError;
use snowbridge_ethereum::{Header, Log, U256};
use sp_runtime::DispatchResult;
use sp_std::prelude::*;

pub mod ringbuffer;
pub mod types;

pub use ringbuffer::{RingBufferMap, RingBufferMapImpl};
pub use polkadot_parachain::primitives::Id as ParaId;
pub use types::{Message, MessageId, MessageNonce, Proof};

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

pub trait SubmitMessage {
	fn submit(source_id: ParaId, handler: u16, payload: &[u8]) -> DispatchResult;
}
