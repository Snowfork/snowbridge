// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use ethabi::{Event, Param, ParamKind, Token};
use snowbridge_core::ParaId;
use snowbridge_ethereum::{log::Log, H160};

use sp_core::{RuntimeDebug, H256};
use sp_std::{convert::TryFrom, prelude::*};

// Used to decode an OutboundMessageAccepted log into an [`Envelope`].
static EVENT_ABI: &Event = &Event {
	signature: "OutboundMessageAccepted(uint256,uint64,bytes32,bytes)",
	inputs: &[
		Param { kind: ParamKind::Uint(256), indexed: true },
		Param { kind: ParamKind::Uint(64), indexed: false },
		Param { kind: ParamKind::FixedBytes(32), indexed: true },
		Param { kind: ParamKind::Bytes, indexed: false },
	],
	anonymous: false,
};

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

use log;

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct EnvelopeDecodeError;

impl TryFrom<Log> for Envelope {
	type Error = EnvelopeDecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
		log::info!(target: "snowbridge-inbound-queue", "FOO -2");

		let tokens = EVENT_ABI.decode(log.topics, log.data).map_err(|_| EnvelopeDecodeError)?;

		log::info!(target: "snowbridge-inbound-queue", "FOO -1");

		let mut iter = tokens.into_iter();

		let dest = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(dest) => dest.low_u32().into(),
			_ => return Err(EnvelopeDecodeError),
		};

		log::info!(target: "snowbridge-inbound-queue", "FOO 0");

		let nonce = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(nonce) => nonce.low_u64(),
			_ => return Err(EnvelopeDecodeError),
		};

		log::info!(target: "snowbridge-inbound-queue", "FOO 1");

		let message_id = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::FixedBytes(message_id) => {
				log::info!(target: "snowbridge-inbound-queue", "FOO 2");
				let message_id: [u8; 32] =
					message_id.try_into().map_err(|_| EnvelopeDecodeError)?;
				log::info!(target: "snowbridge-inbound-queue", "FOO 3");
				H256::from(&message_id)
			},
			_ => return Err(EnvelopeDecodeError),
		};

		log::info!(target: "snowbridge-inbound-queue", "FOO 4");

		let payload = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Bytes(payload) => payload,
			_ => return Err(EnvelopeDecodeError),
		};

		log::info!(target: "snowbridge-inbound-queue", "FOO 5");

		Ok(Self { gateway: log.address, dest, nonce, message_id, payload })
	}
}
