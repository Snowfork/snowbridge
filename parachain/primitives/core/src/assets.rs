
use sp_runtime::{
	traits::MaybeSerializeDeserialize,
	DispatchResult,
};
use frame_support::traits::Imbalance;
use sp_std::{
	cmp::Ord,
};

use frame_support::dispatch::DispatchError;
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
	type PositiveImbalance: Imbalance<U256, Opposite = Self::NegativeImbalance>;
	type NegativeImbalance: Imbalance<U256, Opposite = Self::PositiveImbalance>;

	fn total_issuance() -> U256;

	fn balance(who: &AccountId) -> U256;

	fn burn(amount: U256) -> Self::PositiveImbalance;

	fn issue(amount: U256) -> Self::NegativeImbalance;

	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		amount: U256) -> DispatchResult;

	fn withdraw(
		who: &AccountId,
		amount: U256) -> Result<Self::NegativeImbalance, DispatchError>;

	fn deposit(
		who: &AccountId,
		amount: U256) -> Result<Self::PositiveImbalance, DispatchError>;
}
