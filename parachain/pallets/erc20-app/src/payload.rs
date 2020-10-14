use sp_std::prelude::*;
use ethabi::{Event as ABIEvent, Param, ParamKind, Token};
use artemis_ethereum::{DecodeError, log::Log, H160, U256};

static EVENT_ABI: &ABIEvent = &ABIEvent {
	signature: "AppTransfer(address,bytes32,address,uint256)",
	inputs: &[
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::FixedBytes(32), indexed: false },
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::Uint(256), indexed: false },
	],
	anonymous: false
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Payload<AccountId: codec::Decode> {
	pub sender_addr: H160,
	pub recipient_addr: AccountId,
	pub token_addr: H160,
	pub amount: U256,
}

impl<AccountId: codec::Decode> Payload<AccountId> {

	pub fn decode(payload: Vec<u8>) -> Result<Self, DecodeError> {
		// Decode ethereum Log event from RLP-encoded data
		let log: Log = rlp::decode(&payload)?;
		let tokens = EVENT_ABI.decode(log.topics, log.data)?;
		let mut tokens_iter = tokens.iter();

		let sender_addr = match tokens_iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Address(sender) => *sender,
			_ => return Err(DecodeError::InvalidPayload)
		};

		let recipient_addr = match tokens_iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::FixedBytes(data) => {
				AccountId::decode(&mut data.as_slice()).map_err(|_| DecodeError::InvalidPayload)?
			}
			_ => return Err(DecodeError::InvalidPayload)
		};

		let token_addr = match tokens_iter.next().ok_or(DecodeError::InvalidPayload)? {
			Token::Address(address) => *address,
			_ => return Err(DecodeError::InvalidPayload)
		};

		let amount = match tokens_iter.next().ok_or(DecodeError::InvalidPayload)? {
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
	use super::*;
	use hex_literal::hex;

	const LOG_DATA: [u8; 187] = hex!("
		f8b994c3a1ca063da8d4d3b2c697316ea6e69ccd263a44e1a0be9215fdb4
		23dfc80cce917dc48fa52d3e247875e3d7cea229d3f28661ad0f60b88000
		0000000000000000000000cffeaaf7681c89285d65cfbe808b80e5026965
		73d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5
		6da27d000000000000000000000000f465670390f5214ed43d5027f31ed3
		3764f0448700000000000000000000000000000000000000000000000000
		00000000000002
	");

	#[test]
	fn test_decode() {
		assert_eq!(Payload::decode(LOG_DATA.to_vec()).unwrap(),
			Payload {
				sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
				recipient_addr: hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"],
				token_addr: hex!["f465670390f5214ed43d5027f31ed33764f04487"].into(),
				amount: 2.into()
			}
		);
	}
}
