use sp_std::prelude::*;
use ethabi::{Event as ABIEvent, Param, ParamKind, Token};
use artemis_core::Message;
use artemis_ethereum::{DecodeError, log::Log, H160, U256};

static EVENT_ABI: &ABIEvent = &ABIEvent {
	signature: "AppEvent(uint256,bytes)",
	inputs: &[
		Param { kind: ParamKind::Uint(256), indexed: false },
		Param { kind: ParamKind::Bytes, indexed: false },
	],
	anonymous: false
};

static PAYLOAD_ABI: &[ParamKind] = &[
	// sender address (Ethereum)
	ParamKind::Address,
	// recipient address (Substrate)
	ParamKind::FixedBytes(32),
	// Amount
	ParamKind::Uint(256),
	// Nonce
	ParamKind::Uint(256),
];


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Payload {
	pub sender_addr: H160,
	pub recipient_addr: [u8; 32],
	pub amount: U256,
}

impl Payload {

	pub fn decode(payload: Vec<u8>) -> Result<Self, DecodeError> {
		// Decode ethereum Log event from RLP-encoded data
		let log: Log = rlp::decode(&payload)?;
		let tokens = EVENT_ABI.decode(log.topics, log.data)?;
		let mut tokens_iter = tokens.iter();

		// extract ABI-encoded message payload
		let abi_data = match tokens_iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Bytes(data) => data,
			_ => return Err(DecodeError::InvalidPayload)
		};

		// Tokenize ABI-encoded payload
		let tokens = ethabi::decode(PAYLOAD_ABI, &abi_data)?;
		let mut iter = tokens.iter();

		// Sender address (Ethereum)
		let sender_addr = match iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Address(address) => *address,
			_ => return Err(DecodeError::InvalidPayload)
		};

		// Recipient address (Substrate)
		let recipient_addr: [u8; 32] = match iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::FixedBytes(bytes) => {
				if bytes.len() != 32 {
					return Err(DecodeError::InvalidPayload)
				}
				let mut dst: [u8; 32] = [0; 32];
				dst.copy_from_slice(&bytes);
				dst
			}
			_ => return Err(DecodeError::InvalidPayload)
		};

		// Amount (U256)
		let amount = match iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Uint(amount) => *amount,
			_ => return Err(DecodeError::InvalidPayload)
		};

		Ok(Self {
			sender_addr,
			recipient_addr,
			amount,
		})
	}
}


#[cfg(test)]
mod tests {
	use std::io::prelude::*;
	use std::fs::File;
	use std::io::BufReader;
	use super::*;
	use hex::FromHex;
	use std::path::PathBuf;




	fn to_account_id(hexaddr: &str) -> [u8; 32] {
		let mut buf: [u8; 32] = [0; 32];
		let bytes: Vec<u8> = hexaddr.from_hex().unwrap();
		buf.clone_from_slice(&bytes);
		buf
	}

	#[test]
	fn test_decode() {
		let mut reader = BufReader::new(File::open(fixture_path()).unwrap());
		let mut data: Vec<u8> = Vec::new();
		reader.read_to_end(&mut data).unwrap();

		let log: Log = rlp::decode(&data).unwrap();
		assert_eq!(Event::decode(log).unwrap(),
			Event::SendETH {
				sender: "cffeaaf7681c89285d65cfbe808b80e502696573".parse().unwrap(),
				recipient: to_account_id("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
				amount: 10.into(), nonce: 7
			}
		);
	}
}
