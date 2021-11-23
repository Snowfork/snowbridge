use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{sp_std::prelude::Vec, RuntimeDebug};

use snowbridge_ethereum::U256;

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<AccountId, Data> {
	/// Token owner
	pub owner: AccountId,
	/// Token metadata
	pub metadata: Vec<u8>,
	/// Token Properties
	pub data: Data,
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ERC721TokenData {
	/// The ERC721 smart contract on Ethereum
	pub token_contract: H160,
	/// The ERC721 token id
	pub token_id: U256,
}

pub trait Nft<AccountId, TokenId, TokenData> {
	fn mint(
		owner: &AccountId,
		metadata: Vec<u8>,
		data: TokenData,
	) -> Result<TokenId, DispatchError>;

	fn burn(owner: &AccountId, token_id: TokenId) -> DispatchResult;

	fn transfer(from: &AccountId, to: &AccountId, token_id: TokenId) -> DispatchResult;

	fn is_owner(account: &AccountId, token_id: TokenId) -> bool;

	fn get_token_data(token_id: TokenId) -> Option<TokenInfo<AccountId, TokenData>>;
}
