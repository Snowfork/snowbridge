use ethers::{
	abi::{encode, Token},
	types::{Bytes, H160, U256},
};

pub fn build_native_asset(token: H160, amount: u128) -> Bytes {
	let kind_token = Token::Uint(U256::from(0u8));
	let token_token = Token::Address(token);
	let amount_token = Token::Uint(U256::from(amount));

	encode(&[kind_token, token_token, amount_token]).into()
}
