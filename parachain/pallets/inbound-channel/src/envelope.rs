use codec::{DecodeAll};
use ethabi::{Event, Param, ParamKind, Token};
use snowbridge_ethereum::{log::Log, H160};

use snowbridge_router_primitives::Action;

use sp_core::RuntimeDebug;
use sp_std::{convert::TryFrom, prelude::*};

use xcm::latest::prelude::*;

// Used to decode a raw Ethereum log into an [`Envelope`].
static EVENT_ABI: &Event = &Event {
	signature: "Message(address,address,uint64,bytes)",
	inputs: &[
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::Uint(64), indexed: false },
		Param { kind: ParamKind::Bytes, indexed: false },
	],
	anonymous: false,
};

/// An inbound message that has had its outer envelope decoded.
#[derive(Clone, RuntimeDebug)]
pub struct Envelope {
	/// The address of the outbound channel on Ethereum that forwarded this message.
	pub channel: H160,
	/// The account on Ethereum that authorized the source to send the message.
	pub dest: MultiLocation,
	/// A nonce for enforcing replay protection and ordering.
	pub nonce: u64,
	/// The inner payload generated from the source application.
	pub action: Action,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct EnvelopeDecodeError;

impl TryFrom<Log> for Envelope {
	type Error = EnvelopeDecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
		let tokens = EVENT_ABI.decode(log.topics, log.data).map_err(|_| EnvelopeDecodeError)?;

		let mut iter = tokens.into_iter();

		let dest = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Bytes(dest) => MultiLocation::decode_all(&mut dest.as_ref()).map_err(|_| EnvelopeDecodeError)?,
			_ => return Err(EnvelopeDecodeError),
		};

		let nonce = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(nonce) => nonce.low_u64(),
			_ => return Err(EnvelopeDecodeError),
		};

		let action = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Bytes(payload) => Action::decode_all(&mut payload.as_ref()).map_err(|_| EnvelopeDecodeError)?,
			_ => return Err(EnvelopeDecodeError),
		};

		Ok(Self { channel: log.address, dest, nonce, action })
	}
}
