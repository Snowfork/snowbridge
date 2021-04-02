use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_runtime::RuntimeDebug;

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TokenInfo<AccountId, Data> {
	/// Token owner
	pub owner: AccountId,
	/// Token metadata
	pub metadata: Vec<u8>,
	/// Token Properties
	pub data: Data,
}

pub trait Nft<AccountId, TokenId, TokenData>
{
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
