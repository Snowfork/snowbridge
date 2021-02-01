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
	pub token: H160,
	pub amount: U256,
}

// Message to Ethereum (ABI-encoded)
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OutboundPayload<AccountId: codec::Encode> {
	pub token: H160,
	pub sender: AccountId,
	pub recipient: H160,
	pub amount: U256,
}

impl<AccountId: codec::Encode> OutboundPayload<AccountId> {
	/// ABI-encode this payload
	pub fn encode(&self) -> Vec<u8> {
		let tokens = vec![
			Token::Address(self.token),
			Token::FixedBytes(self.sender.encode()),
			Token::Address(self.recipient),
			Token::Uint(self.amount)
		];
		ethabi::encode_function("unlock(address,bytes32,address,uint256)", tokens.as_ref())
	}
}
