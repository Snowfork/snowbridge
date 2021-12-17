use frame_support::dispatch::DispatchResult;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160, U256};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetId {
	ETH,
	Token(H160),
}

pub trait MultiAsset<AccountId> {
	fn total_issuance(asset_id: AssetId) -> U256;

	fn balance(asset_id: AssetId, who: &AccountId) -> U256;

	fn transfer(
		asset_id: AssetId,
		from: &AccountId,
		to: &AccountId,
		amount: U256,
	) -> DispatchResult;

	fn withdraw(asset_id: AssetId, who: &AccountId, amount: U256) -> DispatchResult;

	fn deposit(asset_id: AssetId, who: &AccountId, amount: U256) -> DispatchResult;
}

pub trait SingleAsset<AccountId> {
	fn asset_id() -> AssetId;

	fn total_issuance() -> U256;

	fn balance(who: &AccountId) -> U256;

	fn transfer(source: &AccountId, dest: &AccountId, amount: U256) -> DispatchResult;

	fn withdraw(who: &AccountId, amount: U256) -> DispatchResult;

	fn deposit(who: &AccountId, amount: U256) -> DispatchResult;
}

pub trait XcmReserveTransfer<AccountId, Origin> {
	fn reserve_transfer(
		origin: Origin,
		asset_id: AssetId,
		para_id: u32,
		recipient: &AccountId,
		amount: U256,
	) -> DispatchResult;
}
