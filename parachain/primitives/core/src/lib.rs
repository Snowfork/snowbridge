//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchError;
use snowbridge_ethereum::Log;
use sp_core::{RuntimeDebug};

use xcm::v3::XcmHash;

pub mod ringbuffer;
pub mod types;

pub use polkadot_parachain::primitives::Id as ParaId;
pub use ringbuffer::{RingBufferMap, RingBufferMapImpl};
pub use types::{Message, MessageId, MessageNonce, Proof};

/// A trait for verifying messages.
///
/// This trait should be implemented by runtime modules that wish to provide message verification
/// functionality.
pub trait Verifier {
	fn verify(message: &Message) -> Result<Log, DispatchError>;
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum SubmitError {
	MessageTooLarge,
}

pub trait OutboundQueue {
	fn validate(
		xcm_hash: XcmHash,
		origin: ParaId,
		handler: u16,
		payload: &[u8],
	) -> Result<(), SubmitError>;

	fn submit(
		xcm_hash: XcmHash,
		origin: ParaId,
		handler: u16,
		payload: &[u8],
	) -> Result<(), SubmitError>;
}
