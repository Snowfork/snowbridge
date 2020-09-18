//! # Asset
//!
//! The asset module provides functionality for handling bridged asset balances.
//!
//! ## Overview
//!
//! This module is used by the ETH and ERC20 pallets to store account balances for bridged assets.
//!
//! Each asset is identified by a unique `H160` hash. This is useful for tracking ERC20 tokens which on Ethereum are identified by a 20-byte contract address.
//!
//! For various reasons, we built our own asset pallet instead of reusing existing work:
//! - `assets`: Too high-level and limited for our needs
//! - `generic-asset`: Its enforced permissions system implies trusted operations. But our system is designed to be trustless.
//! - `balances`: Only stores balances for a single asset. Our ERC20 pallet supports multiple different ERC20 assets.
//!
//! Additionally, we need to store balances using `U256`, which seemed difficult or impossible to plug into the above pallets.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `transfer`: Transferring a balance between accounts.
//!
//! ### Public Functions
//!
//! - `do_mint`: Mint to an account's free balance.
//! - `do_burn`: Burn an account's free balance.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_core::{U256, RuntimeDebug};
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
};

use codec::{Encode, Decode};

use artemis_core::BridgedAssetId;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData {
	pub free: U256
}

type AssetAccountData = AccountData;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Asset {
		pub TotalIssuance: map        hasher(blake2_128_concat) BridgedAssetId => U256;
		pub Account:       double_map hasher(blake2_128_concat) BridgedAssetId, hasher(blake2_128_concat) T::AccountId => AssetAccountData;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Burned(BridgedAssetId, AccountId, U256),
		Minted(BridgedAssetId, AccountId, U256),
		Transferred(BridgedAssetId, AccountId, AccountId, U256),
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

		/// Transfer some free balance to another account.
		// TODO: Calculate weight
		#[weight = 0]
		pub fn transfer(origin, asset_id: BridgedAssetId, to: T::AccountId, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_transfer(asset_id, &who, &to, amount)?;
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	pub fn free_balance(asset_id: BridgedAssetId, who: &T::AccountId) -> U256 {
		<Account<T>>::get(asset_id, who).free
	}

	pub fn do_mint(asset_id: BridgedAssetId, who: &T::AccountId, amount: U256) -> DispatchResult  {
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

	pub fn do_burn(asset_id: BridgedAssetId, who: &T::AccountId, amount: U256) -> DispatchResult  {
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

	fn do_transfer(
		asset_id: BridgedAssetId,
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
				// In theory we'll never hit overflow here since Sum(Account.free) == TotalIssuance.
				to_account.free = to_account.free.checked_add(amount).ok_or(Error::<T>::FreeTransferOverflow)?;
				Ok(())
			})
		})?;
		Self::deposit_event(RawEvent::Transferred(asset_id, from.clone(), to.clone(), amount));
		Ok(())
	}

}
