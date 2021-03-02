use codec::Encode;
use sp_core::RuntimeDebug;
use sp_std::prelude::*;

use artemis_ethereum::H160;
use ethabi::{self, Token};

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
		ethabi::encode_function("mint(bytes32,address,uint128)", tokens.as_ref())
	}
}
