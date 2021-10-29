use codec::Encode;
use sp_core::{RuntimeDebug, U256};
use sp_std::prelude::*;

use ethabi::{self, Token};
use snowbridge_ethereum::H160;

// Message to Ethereum (ABI-encoded)
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OutboundPayload<AccountId: Encode> {
	pub sender: AccountId,
	pub recipient: H160,
	pub amount: U256,
}

impl<AccountId: Encode> OutboundPayload<AccountId> {
	/// ABI-encode this payload
	pub fn encode(&self) -> Vec<u8> {
		let tokens = vec![
			Token::FixedBytes(self.sender.encode()),
			Token::Address(self.recipient),
			Token::Uint(self.amount),
		];
		ethabi::encode_function("mint(bytes32,address,uint256)", tokens.as_ref())
	}
}
