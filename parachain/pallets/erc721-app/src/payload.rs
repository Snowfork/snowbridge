use codec::Encode;
use sp_core::RuntimeDebug;
use sp_std::prelude::*;

use ethabi::{self, Token};
use snowbridge_ethereum::{H160, U256};

// Message to Ethereum (ABI-encoded)
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OutboundPayload<AccountId: Encode> {
	pub token_contract: H160,
	pub token_id: U256,
	pub sender: AccountId,
	pub recipient: H160,
}

impl<AccountId: Encode> OutboundPayload<AccountId> {
	/// ABI-encode this payload
	pub fn encode(&self) -> Vec<u8> {
		let tokens = vec![
			Token::Address(self.token_contract),
			Token::Uint(self.token_id),
			Token::FixedBytes(self.sender.encode()),
			Token::Address(self.recipient),
		];
		ethabi::encode_function("unlock(address,uint256,bytes32,address)", tokens.as_ref())
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
			token_contract: hex!["e1638d0a9f5349bb7d3d748b514b8553dfddb46c"].into(),
			token_id: U256::from_str_radix("100", 10).unwrap(),
			sender: hex!["1aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50"],
			recipient: hex!["ccb3c82493ac988cebe552779e7195a3a9dc651f"].into(),
		};

		println!("Payload:");
		println!("  {:?}", payload);
		println!("Payload (ABI-encoded):");
		println!("  {:?}", payload.encode().to_hex::<String>());
	}
}
