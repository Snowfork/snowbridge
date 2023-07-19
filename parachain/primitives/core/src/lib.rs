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
use sp_core::{RuntimeDebug, H160, H256, U256};
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
	BridgeHalted,
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
	pub command: H256,
	/// ABI-encoded message payload which can be interpreted by the receiving gateway contract
	pub params: Vec<u8>,
}

use ethabi::Token;

const COMMAND_EXECUTE_XCM: H256 = H256::zero();
const COMMAND_CREATE_AGENT: H256 = H256::zero();
const COMMAND_CREATE_CHANNEL: H256 = H256::zero();
const COMMAND_UPGRADE: H256 = H256::zero();

pub enum Command {
	ExecuteXCM { origin_agent_id: H256, payload: Vec<u8> },
	CreateAgent { agent_id: H256 },
	CreateChannel { para_id: ParaId, agent_id: H256 },
	Upgrade { logic: H160, data: Option<Vec<u8>> },
}

impl Command {
	pub fn encode(self) -> (H256, Vec<u8>) {
		match self {
			Command::ExecuteXCM { origin_agent_id, payload } => (
				COMMAND_EXECUTE_XCM,
				ethabi::encode(&vec![
					Token::FixedBytes(origin_agent_id.as_bytes().to_owned()),
					Token::Bytes(payload),
				]),
			),
			Command::CreateAgent { agent_id } => (
				COMMAND_CREATE_AGENT,
				ethabi::encode(&vec![Token::FixedBytes(agent_id.as_bytes().to_owned())]),
			),
			Command::CreateChannel { para_id, agent_id } => {
				let para_id: u32 = para_id.into();
				(
					COMMAND_CREATE_CHANNEL,
					ethabi::encode(&vec![
						Token::Uint(U256::from(para_id)),
						Token::FixedBytes(agent_id.as_bytes().to_owned()),
					]),
				)
			},
			Command::Upgrade { logic, data } => (
				COMMAND_UPGRADE,
				ethabi::encode(&vec![
					Token::Address(logic),
					data.map_or(Token::Bytes(vec![]), |d| Token::Bytes(d)),
				]),
			),
		}
	}
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
