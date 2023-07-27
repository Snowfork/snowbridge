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
use snowbridge_ethereum::Log;
use sp_core::{RuntimeDebug, H160, H256, U256};
use sp_std::{borrow::ToOwned, vec, vec::Vec};

pub mod ringbuffer;
pub mod types;

pub use polkadot_parachain::primitives::Id as ParaId;
pub use ringbuffer::{RingBufferMap, RingBufferMapImpl};
pub use types::{Message, MessageNonce, Proof};

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
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct OutboundMessage {
	/// The parachain from which the message originated
	pub origin: ParaId,
	/// The stable ID for a receiving gateway contract
	pub command: Command,
}

use ethabi::Token;

#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum OperatingMode {
	Normal,
	RejectingOutboundMessages,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum Command {
	AgentExecute { agent_id: H256, command: AgentExecuteCommand },
	Upgrade { impl_address: H160, impl_code_hash: H256, params: Option<Vec<u8>> },
	CreateAgent { agent_id: H256 },
	CreateChannel { para_id: ParaId, agent_id: H256 },
	UpdateChannel { para_id: ParaId, mode: OperatingMode, fee: u128, reward: u128 },
	SetOperatingMode { mode: OperatingMode },
	TransferNativeFromAgent { agent_id: H256, recipient: H160, amount: u128 },
}

impl Command {
	pub fn index(&self) -> u8 {
		match self {
			Command::AgentExecute { .. } => 0,
			Command::Upgrade { .. } => 1,
			Command::CreateAgent { .. } => 2,
			Command::CreateChannel { .. } => 3,
			Command::UpdateChannel { .. } => 4,
			Command::SetOperatingMode { .. } => 5,
			Command::TransferNativeFromAgent { .. } => 6,
		}
	}

	pub fn abi_encode(&self) -> (u8, Vec<u8>) {
		match self {
			Command::AgentExecute { agent_id, command } => (
				self.index(),
				ethabi::encode(&vec![Token::Tuple(vec![
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
					Token::Bytes(command.abi_encode()),
				])]),
			),
			Command::Upgrade { impl_address, impl_code_hash, params } => (
				self.index(),
				ethabi::encode(&vec![Token::Tuple(vec![
					Token::Address(*impl_address),
					Token::FixedBytes(impl_code_hash.as_bytes().to_owned()),
					params.clone().map_or(Token::Bytes(vec![]), |p| Token::Bytes(p)),
				])]),
			),
			Command::CreateAgent { agent_id } => (
				self.index(),
				ethabi::encode(&vec![Token::Tuple(vec![Token::FixedBytes(
					agent_id.as_bytes().to_owned(),
				)])]),
			),
			Command::CreateChannel { para_id, agent_id } => {
				let para_id: u32 = (*para_id).into();
				(
					self.index(),
					ethabi::encode(&vec![Token::Tuple(vec![
						Token::Uint(U256::from(para_id)),
						Token::FixedBytes(agent_id.as_bytes().to_owned()),
					])]),
				)
			},
			Command::UpdateChannel { para_id, mode, fee, reward } => {
				let para_id: u32 = (*para_id).into();
				(
					self.index(),
					ethabi::encode(&vec![Token::Tuple(vec![
						Token::Uint(U256::from(para_id)),
						Token::Uint(U256::from((*mode) as u64)),
						Token::Uint(U256::from(*fee)),
						Token::Uint(U256::from(*reward)),
					])]),
				)
			},
			Command::SetOperatingMode { mode } => (
				self.clone().index(),
				ethabi::encode(&vec![Token::Tuple(vec![Token::Uint(U256::from((*mode) as u64))])]),
			),
			Command::TransferNativeFromAgent { agent_id, recipient, amount } => (
				self.clone().index(),
				ethabi::encode(&vec![Token::Tuple(vec![
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
					Token::Address(*recipient),
					Token::Uint(U256::from(*amount)),
				])]),
			),
		}
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum AgentExecuteCommand {
	TransferToken { token: H160, recipient: H160, amount: u128 },
}

impl AgentExecuteCommand {
	pub fn abi_encode(&self) -> Vec<u8> {
		match self {
			AgentExecuteCommand::TransferToken { token, recipient, amount } => {
				ethabi::encode(&vec![
					Token::Address(*token),
					Token::Address(*recipient),
					Token::Uint(U256::from(*amount)),
				])
			},
		}
	}
}

pub type OutboundMessageHash = H256;

// A trait for enqueueing messages for delivery to Ethereum
pub trait OutboundQueue {
	type Ticket;

	/// Validate a message destined for Ethereum
	fn validate(message: &OutboundMessage) -> Result<Self::Ticket, SubmitError>;

	/// Submit the message for eventual delivery to Ethereum
	fn submit(ticket: Self::Ticket) -> Result<OutboundMessageHash, SubmitError>;
}

impl OutboundQueue for () {
	type Ticket = u64;

	fn validate(message: &OutboundMessage) -> Result<Self::Ticket, SubmitError> {
		Ok(0)
	}

	fn submit(ticket: Self::Ticket) -> Result<OutboundMessageHash, SubmitError> {
		Ok(OutboundMessageHash::zero())
	}
}
