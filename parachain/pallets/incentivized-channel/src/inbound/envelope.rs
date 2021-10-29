use ethabi::{Event, Param, ParamKind, Token};
use snowbridge_ethereum::{log::Log, H160};
use sp_core::RuntimeDebug;
use sp_runtime::traits::Convert;
use sp_std::{convert::TryFrom, prelude::*};

use super::{BalanceOf, Config};

// Used to decode a raw Ethereum log into an [`Envelope`].
static EVENT_ABI: &Event = &Event {
	signature: "Message(address,uint64,uint256,bytes)",
	inputs: &[
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::Uint(64), indexed: false },
		Param { kind: ParamKind::Uint(256), indexed: false },
		Param { kind: ParamKind::Bytes, indexed: false },
	],
	anonymous: false,
};

/// An inbound message that has had its outer envelope decoded.
#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Envelope<T>
where
	T: Config,
{
	/// The address of the outbound channel on Ethereum that forwarded this message.
	pub channel: H160,
	/// The application on Ethereum where the message originated from.
	pub source: H160,
	/// A nonce for enforcing replay protection and ordering.
	pub nonce: u64,
	/// Fee paid by user for relaying the message
	pub fee: BalanceOf<T>,
	/// The inner payload generated from the source application.
	pub payload: Vec<u8>,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct EnvelopeDecodeError;

impl<T> TryFrom<Log> for Envelope<T>
where
	T: Config,
{
	type Error = EnvelopeDecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
		let tokens = EVENT_ABI.decode(log.topics, log.data).map_err(|_| EnvelopeDecodeError)?;

		let mut iter = tokens.into_iter();

		let source = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Address(source) => source,
			_ => return Err(EnvelopeDecodeError),
		};

		let nonce = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(value) => value.low_u64(),
			_ => return Err(EnvelopeDecodeError),
		};

		let fee = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(value) => T::FeeConverter::convert(value).unwrap_or(0u32.into()),
			_ => return Err(EnvelopeDecodeError),
		};

		let payload = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Bytes(payload) => payload,
			_ => return Err(EnvelopeDecodeError),
		};

		Ok(Self { channel: log.address, source, nonce, fee, payload })
	}
}
