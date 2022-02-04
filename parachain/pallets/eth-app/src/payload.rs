use codec::Encode;
use sp_core::RuntimeDebug;
use sp_std::prelude::*;

use ethabi::{self, Token};
use sp_core::H160;

// Message to Ethereum (ABI-encoded)
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OutboundPayload<AccountId: Encode> {
	pub sender: AccountId,
	pub recipient: H160,
	pub amount: u128,
}

impl<AccountId: Encode> OutboundPayload<AccountId> {
	/// ABI-encode this payload
	pub fn encode(&self) -> Vec<u8> {
		let tokens = vec![
			Token::FixedBytes(self.sender.encode()),
			Token::Address(self.recipient),
			Token::Uint(self.amount.into()),
		];
		ethabi::encode_function("unlock(bytes32,address,uint128)", tokens.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex::ToHex;
	use hex_literal::hex;

	#[test]
	fn test_outbound_payload_encode() {
		let payload: OutboundPayload<[u8; 32]> = OutboundPayload {
			sender: hex!["1aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50"],
			recipient: hex!["ccb3c82493ac988cebe552779e7195a3a9dc651f"].into(),
			amount: u128::from_str_radix("1000000000000000000", 10).unwrap(), // 1 ETH
		};

		println!("Payload:");
		println!("  {:?}", payload);
		println!("Payload (ABI-encoded):");
		println!("  {:?}", payload.encode().to_hex::<String>());
	}
}
