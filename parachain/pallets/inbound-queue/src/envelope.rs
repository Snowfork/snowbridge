// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use snowbridge_core::ParaId;
use snowbridge_ethereum::{log::Log, H160};

use sp_core::{RuntimeDebug, H256};
use sp_std::{convert::TryFrom, prelude::*};

use alloy_sol_types::{abi::token::WordToken, sol, SolEvent};

sol! {
	event OutboundMessageAccepted(uint256 indexed destination, uint64 nonce, bytes32 indexed messageID, bytes payload);
}

/// An inbound message that has had its outer envelope decoded.
#[derive(Clone, RuntimeDebug)]
pub struct Envelope {
	/// The address of the outbound queue on Ethereum that emitted this message as an event log
	pub gateway: H160,
	/// The destination parachain.
	pub dest: ParaId,
	/// A nonce for enforcing replay protection and ordering.
	pub nonce: u64,
	/// An id for tracing the message on its route (has no role in bridge consensus)
	pub message_id: H256,
	/// The inner payload generated from the source application.
	pub payload: Vec<u8>,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct EnvelopeDecodeError;

impl TryFrom<Log> for Envelope {
	type Error = EnvelopeDecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
		let topics: Vec<WordToken> = log
			.clone()
			.topics
			.iter()
			.map(|t| WordToken::from(*t.as_fixed_bytes()))
			.collect();

		let event = OutboundMessageAccepted::decode_log(topics, &log.data, true).map_err(|e| {
			log::error!(target: "ethereum-beacon-client","FOO {:?}", e);
			EnvelopeDecodeError
		})?;

		let dest: ParaId = event.destination.saturating_to::<u32>().into();
		let nonce = event.nonce;
		let message_id = H256::from(event.messageID.as_ref());
		let payload = event.payload;

		Ok(Self { gateway: log.address, dest, nonce, message_id, payload })
	}
}
