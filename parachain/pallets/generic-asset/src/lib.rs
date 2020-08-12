#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for PolkaERC20 token assets
///
use sp_std::prelude::*;
use sp_core::{H160, U256, RuntimeDebug};
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
};

use codec::{Encode, Decode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type AssetId = H160;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData {
	pub free: U256
}

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as GenericAsset {
		pub TotalIssuance: map        hasher(blake2_128_concat) AssetId => U256;
		pub Account:       double_map hasher(blake2_128_concat) AssetId, hasher(blake2_128_concat) T::AccountId => AccountData;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Burned(AssetId, AccountId, U256),
		Minted(AssetId, AccountId, U256),
		Transferred(AssetId, AccountId, AccountId, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Free balance got overflowed after transfer.
		FreeTransferOverflow,
		/// Total issuance got overflowed after minting.
		TotalMintingOverflow,
		/// Free balance got overflowed after minting.
		FreeMintingOverflow,
		/// Total issuance got underflowed after burning.
		TotalBurningUnderflow,
		/// Free balance got underflowed after burning.
		FreeBurningUnderflow,
		InsufficientBalance,
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Transfer some liquid free balance to another account.
		#[weight = 0]
		pub fn transfer(origin, asset_id: AssetId, to: T::AccountId, amount: U256) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			Self::do_transfer(asset_id, &origin, &to, amount)?;
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	pub fn free_balance(asset_id: AssetId, who: &T::AccountId) -> U256 {
		<Account<T>>::get(asset_id, who).free
	}

	pub fn do_mint(asset_id: AssetId, who: &T::AccountId, amount: U256) -> DispatchResult  {
		if amount.is_zero() {
			return Ok(())
		}
		<Account<T>>::try_mutate(asset_id, who, |account| -> Result<(), DispatchError> {
			let current_total_issuance = <TotalIssuance>::get(asset_id);
			let new_total_issuance = current_total_issuance.checked_add(amount)
			.ok_or(Error::<T>::TotalMintingOverflow)?;
			account.free = account.free.checked_add(amount)
				.ok_or(Error::<T>::FreeMintingOverflow)?;
			<TotalIssuance>::insert(asset_id, new_total_issuance);
			Ok(())
		})?;
		Self::deposit_event(RawEvent::Minted(asset_id, who.clone(), amount));
		Ok(())
	}

	pub fn do_burn(asset_id: AssetId, who: &T::AccountId, amount: U256) -> DispatchResult  {
		if amount.is_zero() {
			return Ok(())
		}
		<Account<T>>::try_mutate(asset_id, who, |account| -> Result<(), DispatchError> {
			let current_total_issuance = <TotalIssuance>::get(asset_id);
			let new_total_issuance = current_total_issuance.checked_sub(amount)
			.ok_or(Error::<T>::TotalBurningUnderflow)?;
			account.free = account.free.checked_sub(amount)
				.ok_or(Error::<T>::FreeBurningUnderflow)?;
			<TotalIssuance>::insert(asset_id, new_total_issuance);
			Ok(())
		})?;
		Self::deposit_event(RawEvent::Burned(asset_id, who.clone(), amount));
		Ok(())
	}

	pub fn do_transfer(
		asset_id: AssetId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: U256)
	-> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(())
		}
		<Account<T>>::try_mutate(asset_id, from, |from_account| -> DispatchResult {
			<Account<T>>::try_mutate(asset_id, to, |to_account| -> DispatchResult {
				from_account.free = from_account.free.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
				to_account.free = to_account.free.checked_add(amount).ok_or(Error::<T>::FreeTransferOverflow)?;
				Ok(())
			})
		})?;
		Self::deposit_event(RawEvent::Transferred(asset_id, from.clone(), to.clone(), amount));
		Ok(())
	}

}
