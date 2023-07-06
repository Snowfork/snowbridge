// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchError;
use scale_info::TypeInfo;
use snowbridge_ethereum::Log;
use sp_core::{RuntimeDebug, H256};
use sp_std::vec::Vec;

pub mod ringbuffer;
pub mod types;

pub use polkadot_parachain::primitives::Id as ParaId;
pub use ringbuffer::{RingBufferMap, RingBufferMapImpl};
pub use types::{Message, MessageId, MessageNonce, Proof};


/// A stable id for a bridge contract on the Ethereum side
#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, TypeInfo, RuntimeDebug)]
pub struct ContractId([u8; 32]);

impl From<[u8; 32]> for ContractId {
	fn from(value: [u8; 32]) -> Self {
		Self(value)
	}
}

impl From<H256> for ContractId {
	fn from(value: H256) -> Self {
		Self(value.to_fixed_bytes())
	}
}

impl ContractId {
	pub fn to_fixed_bytes(self) -> [u8; 32] {
		self.0
	}
	pub fn as_fixed_bytes(&self) -> &[u8; 32] {
		&self.0
	}
}

impl ContractId {
	pub const fn new(id: [u8; 32]) -> Self {
		Self(id)
	}
}

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

/// A message which can be accepted by the [`OutboundQueue`]
#[derive(Clone, RuntimeDebug)]
pub struct OutboundMessage {
	/// A unique ID to identify a message while its in the processing queue. Usually the Xcm
	/// message hash.
	pub id: H256,
	/// The parachain from which the message originated
	pub origin: ParaId,
	/// The stable ID for a receiving gateway contract
	pub gateway: ContractId,
	/// ABI-encoded message payload which can be interpreted by the receiving gateway contract
	pub payload: Vec<u8>,
}

// A trait for enqueueing messages for delivery to Ethereum
pub trait OutboundQueue {
	type Ticket;

	/// Validate a message destined for Ethereum
	fn validate(message: &OutboundMessage) -> Result<Self::Ticket, SubmitError>;

	/// Submit the message for eventual delivery to Ethereum
	fn submit(ticket: Self::Ticket) -> Result<(), SubmitError>;
}

impl OutboundQueue for () {
	type Ticket = u64;

	fn validate(message: &OutboundMessage) -> Result<Self::Ticket, SubmitError> {
		Ok(0)
	}

	fn submit(ticket: Self::Ticket) -> Result<(), SubmitError> {
		Ok(())
	}
}
