use sp_runtime::{
	traits::MaybeSerializeDeserialize,
	DispatchResult,
};
use sp_std::{
	cmp::Ord,
};

use sp_core::{H160, U256};

pub type AssetId = H160;

pub trait MultiAsset<AccountId>
{
	type AssetId: Copy + MaybeSerializeDeserialize + Ord;

	fn total_issuance(asset_id: Self::AssetId) -> U256;

	fn balance(asset_id: Self::AssetId, who: &AccountId) -> U256;

	fn transfer(
		asset_id: Self::AssetId,
		from: &AccountId,
		to: &AccountId,
		amount: U256) -> DispatchResult;

	fn withdraw(
		asset_id: Self::AssetId,
		who: &AccountId,
		amount: U256) -> DispatchResult;

	fn deposit(
		asset_id: Self::AssetId,
		who: &AccountId,
		amount: U256) -> DispatchResult;
}

pub trait Asset<AccountId>
{
	fn total_issuance() -> U256;

	fn balance(who: &AccountId) -> U256;

	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		amount: U256) -> DispatchResult;

	fn withdraw(
		who: &AccountId,
		amount: U256) -> DispatchResult;

	fn deposit(
		who: &AccountId,
		amount: U256) -> DispatchResult;
}
