use sp_core::RuntimeDebug;
use sp_std::prelude::*;
use codec::{Encode, Decode};

use ethabi::{self, Token};
use artemis_ethereum::{H160, U256};

// Message from Ethereum (SCALE-encoded)
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct InboundPayload<AccountId: codec::Decode> {
	pub sender: H160,
	pub recipient: AccountId,
	pub amount: U256,
}

// Message to Ethereum (ABI-encoded)
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OutboundPayload<AccountId: codec::Encode> {
	pub sender: AccountId,
	pub recipient: H160,
	pub amount: U256,
}

impl<AccountId: codec::Encode> OutboundPayload<AccountId> {
	/// ABI-encode this payload
	pub fn encode(&self) -> Vec<u8> {
		let tokens = vec![
			Token::FixedBytes(self.sender.encode()),
			Token::Address(self.recipient),
			Token::Uint(self.amount)
		];
		ethabi::encode_function("unlock(bytes32,address,uint256)", tokens.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;
	use hex::ToHex;

	const INBOUND_PAYLOAD_BYTES: [u8; 84] = hex!("
		1ed28b61269a6d3d28d07b1fd834ebe4e703368ed43593c715fdd31c61141abd
		04a99fd6822c8558854ccde39a5684e7a56da27d000100000000000000000000
		0000000000000000000000000000000000000000
	");

	#[test]
	fn test_inbound_payload_decode() {
		assert_eq!(
			InboundPayload::decode(&mut &INBOUND_PAYLOAD_BYTES[..]).unwrap(),
			InboundPayload {
				sender: hex!["1ed28b61269a6d3d28d07b1fd834ebe4e703368e"].into(),
				recipient: hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"],
				amount: 256.into()
			}
		);
	}

	#[test]
	fn test_outbound_payload_encode() {
		let payload: OutboundPayload<[u8; 32]> = OutboundPayload {
			sender: hex!["1aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50"],
			recipient: hex!["ccb3c82493ac988cebe552779e7195a3a9dc651f"].into(),
			amount: U256::from_str_radix("1000000000000000000", 10).unwrap(), // 1 ETH
		};

		println!("Payload:");
		println!("  {:?}", payload);
		println!("Payload (ABI-encoded):");
		println!("  {:?}", payload.encode().to_hex::<String>());
	}

}
