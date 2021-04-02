use frame_support::dispatch::{DispatchError, DispatchResult};

pub trait Nft<AccountId, TokenId, TokenData>
{
	fn mint(
		owner: &AccountId,
		metadata: Vec<u8>,
		data: TokenData,
	) -> Result<TokenId, DispatchError>;

	fn burn(owner: &AccountId, token: TokenId) -> DispatchResult;

	fn transfer(from: &AccountId, to: &AccountId, token: TokenId) -> DispatchResult;

	fn is_owner(account: &AccountId, token: TokenId) -> bool;
}
