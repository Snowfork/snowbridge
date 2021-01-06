use ethabi::{Event as ABIEvent, Param, ParamKind, Token};
use artemis_ethereum::{DecodeError, log::Log, H160, U256};

use sp_core::RuntimeDebug;
use sp_std::prelude::*;
use sp_std::convert::TryFrom;

static EVENT_ABI: &ABIEvent = &ABIEvent {
	signature: "AppTransfer(address,bytes32,uint256)",
	inputs: &[
		Param { kind: ParamKind::Address, indexed: false },
		Param { kind: ParamKind::FixedBytes(32), indexed: false },
		Param { kind: ParamKind::Uint(256), indexed: false },
	],
	anonymous: false
};

#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct InPayload<AccountId: codec::Decode> {
	pub sender_addr: H160,
	pub recipient_addr: AccountId,
	pub amount: U256,
}

impl<AccountId: codec::Decode> TryFrom<Log> for InPayload<AccountId>{
	type Error = DecodeError;

	fn try_from(log: Log) -> Result<Self, Self::Error> {
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

		let amount = match tokens_iter.next().ok_or(DecodeError::InvalidPayload)? {
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

// Message to Ethereum
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OutPayload<AccountId: codec::Encode> {
	pub sender_addr: AccountId,
	pub recipient_addr: H160,
	pub amount: U256,
}

impl<AccountId: codec::Encode> OutPayload<AccountId> {
	/// ABI-encode this payload
	pub fn encode(&self) -> Vec<u8> {
		let tokens = vec![
			Token::FixedBytes(self.sender_addr.encode()),
			Token::Address(self.recipient_addr),
			Token::Uint(self.amount)
		];
		ethabi::encode(tokens.as_ref())
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
	fn test_from_log_conversion() {
		let log: Log = rlp::decode(&LOG_DATA).unwrap();
	
		assert_eq!(
			InPayload::try_from(log).unwrap(),
			InPayload {
				sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
				recipient_addr: hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"],
				amount: U256::from_dec_str("1000000000000000").unwrap(),
			}
		);
	}
}
