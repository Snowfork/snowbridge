use ethabi::{Event, Param, ParamKind, Token};
use snowbridge_ethereum::{log::Log, H160};

use sp_core::RuntimeDebug;
use sp_std::{convert::TryFrom, prelude::*};

// Used to decode a raw Ethereum log into an [`Envelope`].
static EVENT_ABI: &Event = &Event {
	signature: "Message(address,uint64,bytes)",
	inputs: &[
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::Uint(64), indexed: false },
		Param { kind: ParamKind::Bytes, indexed: false },
	],
	anonymous: false,
};

/// An inbound message that has had its outer envelope decoded.
#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Envelope {
	/// The address of the outbound channel on Ethereum that forwarded this message.
	pub channel: H160,
	/// The application on Ethereum where the message originated from.
	pub source: H160,
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

		let source = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Address(source) => source,
			_ => return Err(EnvelopeDecodeError),
		};

		let nonce = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(value) => value.low_u64(),
			_ => return Err(EnvelopeDecodeError),
		};

		let payload = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Bytes(payload) => payload,
			_ => return Err(EnvelopeDecodeError),
		};

		Ok(Self { channel: log.address, source, nonce, payload })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;

	const LOG: [u8; 284] = hex!(
		"
		f901199430d2da52e36f80b17fe2694a5e4900b81cf26344e1a0779b38144a38
		cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15bb8e000000000
		0000000000000000abe98e5ef4dc7a5c4f317823986fe48649f0edbb00000000
		0000000000000000000000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000006000000000
		000000000000000000000000000000000000000000000000000000541ed28b61
		269a6d3d28d07b1fd834ebe4e703368ed43593c715fdd31c61141abd04a99fd6
		822c8558854ccde39a5684e7a56da27d00010000000000000000000000000000
		00000000000000000000000000000000000000000000000000000000
	"
	);

	#[test]
	fn test_try_from_log() {
		let log: Log = rlp::decode(&LOG).unwrap();
		let envelope = Envelope::try_from(log).unwrap();

		assert_eq!(
			envelope,
			Envelope {
				channel: hex!["30d2da52e36f80b17fe2694a5e4900b81cf26344"].into(),
				source: hex!["abe98e5ef4dc7a5c4f317823986fe48649f0edbb"].into(),
				nonce: 0,
				payload: hex!(
					"
					1ed28b61269a6d3d28d07b1fd834ebe4e703368ed43593c715fdd31c61141abd
					04a99fd6822c8558854ccde39a5684e7a56da27d000100000000000000000000
					0000000000000000000000000000000000000000"
				)
				.into(),
			}
		)
	}
}
