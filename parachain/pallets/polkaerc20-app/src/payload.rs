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
	// sender address
	ParamKind::Address,
	// recipient address (Substrate)
	ParamKind::FixedBytes(32),
	// Token contract address
	ParamKind::Address,
	// Amount
	ParamKind::Uint(256),
	// Nonce
	ParamKind::Uint(256),
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Payload {
	pub sender_addr: H160,
	pub recipient_addr: [u8; 32],
	pub token_addr: H160,
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

		// Sender
		let sender_addr = match iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Address(address) => *address,
			_ => return Err(DecodeError::InvalidPayload)
		};

		// Recipient
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

		// Token address
		let token_addr = match iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Address(address) => *address,
			_ => return Err(DecodeError::InvalidPayload)
		};

		// Amount
		let amount = match iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Uint(amount) => *amount,
			_ => return Err(DecodeError::InvalidPayload)
		};

		Ok(Self {
			sender_addr,
			recipient_addr,
			token_addr,
			amount,
		})
	}
}


#[cfg(test)]
mod tests {
	use std::io::prelude::*;
	use std::io::BufReader;
	use super::*;
	use hex::FromHex;
	use hex_literal::hex;

	const LOG_DATA: [u8; 317] = hex!("
		f9013a940d27b0069241c03575669fed1badcbccdc0dd4d1e1a06bafbf13
		bfcea5e4ce5cd1a03b246069acefcd0bada5ef4e1a059b37a08c2399b901
		000000000000000000000000000000000000000000000000000000000000
		000000000000000000000000000000000000000000000000000000000000
		000000004000000000000000000000000000000000000000000000000000
		000000000000a0000000000000000000000000cffeaaf7681c89285d65cf
		be808b80e5026965738eaf04151687736326c9fea17e25fc5287613693c9
		12909cb226aa4794f26a4800000000000000000000000000000000000000
		000000000000000000000000000000000000000000000000000000000000
		00000000000000000000000000000a000000000000000000000000000000
		0000000000000000000000000000000007
	");

	#[test]
	fn test_decode() {
		let mut reader = BufReader::new(File::open(fixture_path()).unwrap());
		let mut data: Vec<u8> = Vec::new();
		reader.read_to_end(&mut data).unwrap();


		let log: Log = rlp::decode(&data).unwrap();
		assert_eq!(Payload::decode(log).unwrap(),
			Payload {
				sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
				recipient_addr: hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"],
				token_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
				amount: 10.into()
			}
		);
	}
}
