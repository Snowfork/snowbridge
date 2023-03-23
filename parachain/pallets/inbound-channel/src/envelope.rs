use ethabi::{Event, Param, ParamKind, Token};
use snowbridge_ethereum::{log::Log, H160};

use sp_core::RuntimeDebug;
use sp_std::{convert::TryFrom, prelude::*};

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
#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Envelope {
	/// The address of the outbound channel on Ethereum that forwarded this message.
	pub channel: H160,
	/// The application on Ethereum where the message originated from.
	pub source: H160,
	/// The account on Ethereum that authorized the source to send the message.
	pub account: H160,
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

		let account = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Address(account) => account,
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

		Ok(Self { channel: log.address, account, source, nonce, payload })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;

	const LOG: [u8; 251] = hex!(
		"
		f8f99486d9ac0bab011917f57b9e9607833b4340f9d4f8e1a0daab80e8986999
		7d1cabbe1122788e90fe72b9234ff97a9217dcbb5126f3562fb8c00000000000
		0000000000000089b4ab1ef20763630df9743acf155865600daff20000000000
		0000000000000004e00e6d2e9ea1e2af553de02a5172120bfa5c3e0000000000
		0000000000000000000000000000000000000000000000000000010000000000
		0000000000000000000000000000000000000000000000000000800000000000
		0000000000000000000000000000000000000000000000000000206172626974
		726172792d7061796c6f6164000000000000000000000000000000
	"
	);

	#[test]
	fn test_try_from_log() {
		let log: Log = rlp::decode(&LOG).unwrap();
		let envelope = Envelope::try_from(log).unwrap();

		assert_eq!(
			envelope,
			Envelope {
				channel: hex!["86d9ac0bab011917f57b9e9607833b4340f9d4f8"].into(),
				source: hex!["89b4ab1ef20763630df9743acf155865600daff2"].into(),
				account: hex!["04e00e6d2e9ea1e2af553de02a5172120bfa5c3e"].into(),
				nonce: 1,
				payload: hex!("6172626974726172792d7061796c6f6164000000000000000000000000000000")
					.into(),
			}
		)
	}
}
