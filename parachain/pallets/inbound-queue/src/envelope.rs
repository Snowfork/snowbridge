// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use snowbridge_core::{ChannelId, ParaId};

use sp_core::{RuntimeDebug, H160, H256};
use sp_std::{convert::TryFrom, prelude::*};

use alloy_primitives::{Address, Bytes, FixedBytes, B256};
use alloy_rlp::RlpDecodable;
use alloy_sol_types::{sol, SolEvent};

#[derive(RlpDecodable, RuntimeDebug)]
pub struct Log {
	pub address: Address,
	pub topics: Vec<B256>,
	pub data: Bytes,
}

sol! {
	event OutboundMessageAccepted(bytes32 indexed channelID, uint64 nonce, bytes32 indexed messageID, bytes payload);
}

/// An inbound message that has had its outer envelope decoded.
#[derive(Clone, RuntimeDebug)]
pub struct Envelope {
	/// The address of the outbound queue on Ethereum that emitted this message as an event log
	pub gateway: H160,
	/// The message Channel
	pub channel_id: ChannelId,
	/// A nonce for enforcing replay protection and ordering.
	pub nonce: u64,
	/// An id for tracing the message on its route (has no role in bridge consensus)
	pub message_id: H256,
	/// The inner payload generated from the source application.
	pub payload: Vec<u8>,
}

#[derive(Copy, Clone, RuntimeDebug)]
pub struct EnvelopeDecodeError;

impl TryFrom<Log> for Envelope {
	type Error = EnvelopeDecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
		let event = OutboundMessageAccepted::decode_log(log.topics, &log.data, true)
			.map_err(|_| EnvelopeDecodeError)?;

		let channel_id = &[u8; 32] = event.channelID.into();

		event.channelID.Ok(Self {
			gateway: H160::from(log.address.as_ref()),
			channel_id: ChannelID::from(channel_id),
			nonce: event.nonce,
			message_id: H256::from(event.messageID.as_ref()),
			payload: event.payload,
		})
	}
}
