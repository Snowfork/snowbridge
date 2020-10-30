//! # Asset
//!
//! The asset module provides functionality for handling account balances.
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
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	traits::{Imbalance},
	dispatch::{DispatchResult, DispatchError},
	Parameter
};

use sp_core::{U256};
use sp_runtime::traits::{MaybeSerializeDeserialize, Member};

use artemis_core::assets::{MultiAsset, Asset};
use sp_std::{marker, result};

mod imbalances;
pub use crate::imbalances::{NegativeImbalance, PositiveImbalance};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type AssetId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;
}

decl_storage! {
	trait Store for Module<T: Trait> as Asset {
		pub TotalIssuance get(fn total_issuance): map hasher(blake2_128_concat) T::AssetId => U256;
		pub Balances get(fn balances): double_map hasher(blake2_128_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => U256;
	}
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId,
		<T as Trait>::AssetId,	{
		Transferred(AssetId, AccountId, AccountId, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		TotalIssuanceOverflow,
		TotalIssuanceUnderflow,
		BalanceOverflow,
		InsufficientBalance
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Transfer some free balance to another account.
		#[weight = 10]
		pub fn transfer(origin,
						dest: T::AccountId,
						asset_id: T::AssetId,
						amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<Self as MultiAsset<_>>::transfer(asset_id, &who, &dest, amount)
		}

	}
}

impl<T: Trait> Module<T> {

}

impl<T: Trait> MultiAsset<T::AccountId> for Module<T> {

	type AssetId = T::AssetId;

	fn total_issuance(asset_id: Self::AssetId) -> U256 {
		Module::<T>::total_issuance(asset_id)
	}

	fn balance(asset_id: Self::AssetId, who: &T::AccountId) -> U256 {
		Module::<T>::balances(asset_id, who)
	}

	fn deposit(asset_id: Self::AssetId, who: &T::AccountId, amount: U256) -> DispatchResult  {
		if amount.is_zero() {
			return Ok(())
		}
		<Balances<T>>::try_mutate(asset_id, who, |balance| -> Result<(), DispatchError> {
			let current_total_issuance = Self::total_issuance(asset_id);
			let new_total_issuance = current_total_issuance.checked_add(amount)
				.ok_or(Error::<T>::TotalIssuanceOverflow)?;
			*balance = balance.checked_add(amount)
				.ok_or(Error::<T>::BalanceOverflow)?;
			<TotalIssuance<T>>::insert(asset_id, new_total_issuance);
			Ok(())
		})
	}

	fn withdraw(asset_id: Self::AssetId, who: &T::AccountId, amount: U256) -> DispatchResult  {
		if amount.is_zero() {
			return Ok(())
		}
		<Balances<T>>::try_mutate(asset_id, who, |balance| -> Result<(), DispatchError> {
			let current_total_issuance = Self::total_issuance(asset_id);
			let new_total_issuance = current_total_issuance.checked_sub(amount)
				.ok_or(Error::<T>::TotalIssuanceUnderflow)?;
			*balance = balance.checked_sub(amount)
				.ok_or(Error::<T>::InsufficientBalance)?;
			<TotalIssuance<T>>::insert(asset_id, new_total_issuance);
			Ok(())
		})
	}

	fn transfer(
		asset_id: Self::AssetId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: U256)
	-> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(())
		}
		<Balances<T>>::try_mutate(asset_id, from, |from_balance| -> DispatchResult {
			<Balances<T>>::try_mutate(asset_id, to, |to_balance| -> DispatchResult {
				*from_balance = from_balance.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
				*to_balance = to_balance.checked_add(amount).ok_or(Error::<T>::BalanceOverflow)?;
				Ok(())
			})
		})
	}
}


pub struct AssetAdaptor<T, GetAssetId>(marker::PhantomData<(T, GetAssetId)>);

impl<T, GetAssetId> Asset<T::AccountId> for AssetAdaptor<T, GetAssetId>
where
	T: Trait,
	GetAssetId: frame_support::traits::Get<T::AssetId>,
{
	type PositiveImbalance = PositiveImbalance<T, GetAssetId>;
	type NegativeImbalance = NegativeImbalance<T, GetAssetId>;

	fn total_issuance() -> U256 {
		Module::<T>::total_issuance(GetAssetId::get())
	}

	fn balance(who: &T::AccountId) -> U256 {
		Module::<T>::balances(GetAssetId::get(), who)
	}

	fn burn(mut amount: U256) -> Self::PositiveImbalance {
		if amount.is_zero() {
			return PositiveImbalance::zero();
		}
		<TotalIssuance<T>>::mutate(GetAssetId::get(), |issued| {
			*issued = issued.checked_sub(amount).unwrap_or_else(|| {
				amount = *issued;
				U256::zero()
			});
		});
		PositiveImbalance::new(amount)
	}

	fn issue(mut amount: U256) -> Self::NegativeImbalance {
		if amount.is_zero() {
			return NegativeImbalance::zero();
		}
		<TotalIssuance<T>>::mutate(GetAssetId::get(), |issued| {
			*issued = issued.checked_add(amount).unwrap_or_else(|| {
				amount = U256::max_value() - *issued;
				U256::max_value()
			})
		});
		NegativeImbalance::new(amount)
	}

	fn transfer(
		source: &T::AccountId,
		dest: &T::AccountId,
		amount: U256,
	) -> DispatchResult {
		<Module<T> as MultiAsset<_>>::transfer(GetAssetId::get(), source, dest, amount)
	}

	fn deposit(
		who: &T::AccountId,
		value: U256,
	) -> result::Result<Self::PositiveImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::PositiveImbalance::zero());
		}
		let asset_id = GetAssetId::get();
		let balance = <Balances<T>>::get(asset_id, who)
			.checked_add(value)
			.ok_or(Error::<T>::BalanceOverflow)?;
		<Balances<T>>::insert(asset_id, who, balance);

		Ok(Self::PositiveImbalance::new(value))
	}

	fn withdraw(
		who: &T::AccountId,
		value: U256,
	) -> result::Result<Self::NegativeImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::NegativeImbalance::zero());
		}
		let asset_id = GetAssetId::get();
		let balance = <Balances<T>>::get(asset_id, who)
			.checked_sub(value)
			.ok_or(Error::<T>::InsufficientBalance)?;

		<Balances<T>>::insert(asset_id, who, balance);

		Ok(Self::NegativeImbalance::new(value))
	}
}
