// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use ethabi::{Event, Param, ParamKind, Token};
use snowbridge_core::ParaId;
use snowbridge_ethereum::{log::Log, H160};

use sp_core::RuntimeDebug;
use sp_std::{convert::TryFrom, prelude::*};

// Used to decode a raw Ethereum log into an [`Envelope`].
static EVENT_ABI: &Event = &Event {
	signature: "Message(uint32,uint64,bytes)",
	inputs: &[
		Param { kind: ParamKind::Uint(32), indexed: true },
		Param { kind: ParamKind::Uint(64), indexed: true },
		Param { kind: ParamKind::Bytes, indexed: false },
	],
	anonymous: false,
};

/// An inbound message that has had its outer envelope decoded.
#[derive(Clone, RuntimeDebug)]
pub struct Envelope {
	/// The address of the outbound queue on Ethereum that emitted this message as an event log
	pub outbound_queue_address: H160,
	/// The destination parachain.
	pub dest: ParaId,
	/// A nonce for enforcing replay protection and ordering.
	pub nonce: u64,
	/// The inner payload generated from the source application.
	pub payload: Vec<u8>,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct EnvelopeDecodeError;

impl TryFrom<Log> for Envelope {
	type Error = EnvelopeDecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
		let tokens = EVENT_ABI.decode(log.topics, log.data).map_err(|_| EnvelopeDecodeError)?;

		let mut iter = tokens.into_iter();

		let dest = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(dest) => dest.low_u32().into(),
			_ => return Err(EnvelopeDecodeError),
		};

		let nonce = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(nonce) => nonce.low_u64(),
			_ => return Err(EnvelopeDecodeError),
		};

		let payload = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Bytes(payload) => payload,
			_ => return Err(EnvelopeDecodeError),
		};

		Ok(Self { outbound_queue_address: log.address, dest, nonce, payload })
	}
}
