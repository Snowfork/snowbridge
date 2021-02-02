use frame_support::dispatch::DispatchResult;

use codec::{Encode, Decode};
use sp_core::{RuntimeDebug, H160, U256};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetId {
	ETH,
	Token(H160)
}

pub trait MultiAsset<AccountId>
{
	fn total_issuance(asset_id: AssetId) -> U256;

	fn balance(asset_id: AssetId, who: &AccountId) -> U256;

	fn transfer(
		asset_id: AssetId,
		from: &AccountId,
		to: &AccountId,
		amount: U256) -> DispatchResult;

	fn withdraw(
		asset_id: AssetId,
		who: &AccountId,
		amount: U256) -> DispatchResult;

	fn deposit(
		asset_id: AssetId,
		who: &AccountId,
		amount: U256) -> DispatchResult;
}

pub trait SingleAsset<AccountId>
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
