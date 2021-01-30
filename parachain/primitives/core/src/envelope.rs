use ethabi::{Event, Param, ParamKind, Token};
use artemis_ethereum::{log::Log, H160};

use sp_core::RuntimeDebug;
use sp_std::prelude::*;
use sp_std::convert::TryFrom;

static EVENT_ABI: &Event = &Event {
	signature: "Message(address,uint64,bytes)",
	inputs: &[
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::Uint(64), indexed: false },
		Param { kind: ParamKind::Bytes, indexed: false },
	],
	anonymous: false
};

#[derive(Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Envelope {
	pub channel: H160,
	pub source: H160,
	pub nonce: u64,
	pub payload: Vec<u8>,
}

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct EnvelopeDecodeError;

impl TryFrom<Log> for Envelope {
	type Error = EnvelopeDecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
		let tokens = EVENT_ABI.decode(log.topics, log.data)
			.map_err(|_| EnvelopeDecodeError)?;

		let mut iter = tokens.into_iter();

		let source = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Address(source) => source,
			_ => return Err(EnvelopeDecodeError)
		};

		let nonce = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Uint(value) => {
				// this should be safe since in Solidity, a uint64 fits
				// into the lower 64-bits of a uint256.
				value.low_u64()
			}
			_ => return Err(EnvelopeDecodeError)
		};

		let payload = match iter.next().ok_or(EnvelopeDecodeError)? {
			Token::Bytes(payload) => payload,
			_ => return Err(EnvelopeDecodeError)
		};

		Ok(Self {
			channel: log.address,
			source,
			nonce,
			payload,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;

	const LOG_DATA: [u8; 155] = hex!("
		f899947c5c2fb581612f040ebf9e74f94c9eac8681a95fe1a0691df88ac0
		2f64f3b39fb1b52b940a2730e41ae20f39eec131634df2f8edce77b86000
		0000000000000000000000cffeaaf7681c89285d65cfbe808b80e5026965
		73d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5
		6da27d00000000000000000000000000000000000000000000000000038d
		7ea4c68000
	");

	#[test]
	fn test_try_from_log() {

	}
}
