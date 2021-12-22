//! # Assets
//!
//! The Assets module provides functionality for handling fungible assets.
//!
//! - [`assets::Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//
//! ## Overview
//!
//! The assets module is used by the Polkadot-Ethereum bridge to store ETH and ERC20 token balances.
//!
//! ### Implementations
//!
//! The Assets module provides implementations for the following traits.
//!
//! - [`MultiAsset`](../snowbridge_core/assets/trait.MultiAsset.html): Functions for dealing with a
//! multiple fungible assets.
//! - [`SingleAsset`](../snowbridge_core/assets/trait.SingleAsset.html): Functions for dealing with
//!   a
//! single fungible asset.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer`: Transferring a balance between accounts.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	traits::Get,
};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;

use sp_core::U256;
use sp_runtime::traits::StaticLookup;

use snowbridge_core::assets::{AssetId, MultiAsset, SingleAsset};
use sp_std::marker;
pub use weights::WeightInfo;

mod benchmarking;

pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
	/// Weight information for extrinsics in this pallet
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as Assets {
		pub TotalIssuance get(fn total_issuance): map hasher(blake2_128_concat) AssetId => U256;
		pub Balances get(fn balances): double_map hasher(blake2_128_concat) AssetId, hasher(blake2_128_concat) T::AccountId => U256;
	}
	add_extra_genesis {
		config(balances): Vec<(AssetId, T::AccountId, U256)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref asset_id, ref who, amount) in config.balances.iter() {
				let total_issuance = TotalIssuance::get(asset_id);
				TotalIssuance::insert(asset_id, total_issuance + amount);
				Balances::<T>::insert(asset_id, who, amount);
			}
		});
	}
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Config>::AccountId,
	{
		Transferred(AssetId, AccountId, AccountId, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		TotalIssuanceOverflow,
		TotalIssuanceUnderflow,
		BalanceOverflow,
		InsufficientBalance
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Transfer some free balance to another account.
		#[weight = T::WeightInfo::transfer()]
		pub fn transfer(origin,
						asset_id: AssetId,
						dest: <T::Lookup as StaticLookup>::Source,
						amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			<Self as MultiAsset<_>>::transfer(asset_id, &who, &dest, amount)
		}
	}
}

impl<T: Config> MultiAsset<T::AccountId> for Module<T> {
	fn total_issuance(asset_id: AssetId) -> U256 {
		Module::<T>::total_issuance(asset_id)
	}

	fn balance(asset_id: AssetId, who: &T::AccountId) -> U256 {
		Module::<T>::balances(asset_id, who)
	}

	fn deposit(asset_id: AssetId, who: &T::AccountId, amount: U256) -> DispatchResult {
		if amount.is_zero() {
			return Ok(())
		}
		<Balances<T>>::try_mutate(asset_id, who, |balance| -> Result<(), DispatchError> {
			let current_total_issuance = Self::total_issuance(asset_id);
			let new_total_issuance = current_total_issuance
				.checked_add(amount)
				.ok_or(Error::<T>::TotalIssuanceOverflow)?;
			*balance = balance.checked_add(amount).ok_or(Error::<T>::BalanceOverflow)?;
			<TotalIssuance>::insert(asset_id, new_total_issuance);
			Ok(())
		})
	}

	fn withdraw(asset_id: AssetId, who: &T::AccountId, amount: U256) -> DispatchResult {
		if amount.is_zero() {
			return Ok(())
		}
		<Balances<T>>::try_mutate(asset_id, who, |balance| -> Result<(), DispatchError> {
			let current_total_issuance = Self::total_issuance(asset_id);
			let new_total_issuance = current_total_issuance
				.checked_sub(amount)
				.ok_or(Error::<T>::TotalIssuanceUnderflow)?;
			*balance = balance.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
			<TotalIssuance>::insert(asset_id, new_total_issuance);
			Ok(())
		})
	}

	fn transfer(
		asset_id: AssetId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: U256,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(())
		}
		<Balances<T>>::try_mutate(asset_id, from, |from_balance| -> DispatchResult {
			<Balances<T>>::try_mutate(asset_id, to, |to_balance| -> DispatchResult {
				*from_balance =
					from_balance.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
				*to_balance = to_balance.checked_add(amount).ok_or(Error::<T>::BalanceOverflow)?;
				Ok(())
			})
		})
	}
}

pub struct SingleAssetAdaptor<T, I>(marker::PhantomData<(T, I)>);

impl<T, I> SingleAsset<T::AccountId> for SingleAssetAdaptor<T, I>
where
	T: Config,
	I: Get<AssetId>,
{
	fn asset_id() -> AssetId {
		I::get()
	}

	fn total_issuance() -> U256 {
		Module::<T>::total_issuance(I::get())
	}

	fn balance(who: &T::AccountId) -> U256 {
		Module::<T>::balances(I::get(), who)
	}

	fn deposit(who: &T::AccountId, amount: U256) -> DispatchResult {
		<Module<T> as MultiAsset<_>>::deposit(I::get(), who, amount)
	}

	fn withdraw(who: &T::AccountId, amount: U256) -> DispatchResult {
		<Module<T> as MultiAsset<_>>::withdraw(I::get(), who, amount)
	}

	fn transfer(source: &T::AccountId, dest: &T::AccountId, amount: U256) -> DispatchResult {
		<Module<T> as MultiAsset<_>>::transfer(I::get(), source, dest, amount)
	}
}
