//! # ETH
//!
//! An application that implements a bridged ETH asset.
//!
//! ## Overview
//!
//! ETH balances are stored in the tightly-coupled [`asset`] runtime module. When an account holder
//! burns some of their balance, a `Transfer` event is emitted. An external relayer will listen for
//! this event and relay it to the other chain.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `burn`: Burn an ETH balance.
#![cfg_attr(not(feature = "std"), no_std)]

mod payload;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	log,
	traits::{fungible::Mutate, EnsureOrigin},
	transactional, PalletId,
};
use frame_system::ensure_signed;
use sp_core::H160;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use snowbridge_core::{
	assets::{RemoteParachain, XcmReserveTransfer},
	ChannelId, OutboundRouter,
};

pub use pallet::*;
use payload::OutboundPayload;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		type Asset: Mutate<Self::AccountId, Balance = u128>;

		type OutboundRouter: OutboundRouter<Self::AccountId>;

		type CallOrigin: EnsureOrigin<Self::Origin, Success = H160>;

		type WeightInfo: WeightInfo;

		type XcmReserveTransfer: XcmReserveTransfer<Self::AccountId, Self::Origin>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Burned(T::AccountId, H160, u128),
		Minted(H160, T::AccountId, u128),
	}

	#[pallet::storage]
	#[pallet::getter(fn address)]
	pub(super) type Address<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub address: H160,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { address: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<Address<T>>::put(self.address);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Users can burn their holdings to release funds on the Ethereum side
		#[pallet::weight({
			match channel_id {
				ChannelId::Basic => T::WeightInfo::burn_basic_channel(),
				ChannelId::Incentivized => T::WeightInfo::burn_incentivized_channel(),
			}
		})]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			channel_id: ChannelId,
			recipient: H160,
			amount: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Asset::burn_from(&who, amount)?;

			let message =
				OutboundPayload { sender: who.clone(), recipient: recipient.clone(), amount };

			T::OutboundRouter::submit(channel_id, &who, <Address<T>>::get(), &message.encode())?;
			Self::deposit_event(Event::Burned(who.clone(), recipient, amount));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			sender: H160,
			recipient: <T::Lookup as StaticLookup>::Source,
			amount: u128,
			destination: Option<RemoteParachain>,
		) -> DispatchResult {
			let who = T::CallOrigin::ensure_origin(origin.clone())?;
			if who != <Address<T>>::get() {
				return Err(DispatchError::BadOrigin.into())
			}

			let recipient = T::Lookup::lookup(recipient)?;
			T::Asset::mint_into(&recipient, amount)?;
			Self::deposit_event(Event::Minted(sender, recipient.clone(), amount));

			if let Some(destination) = destination {
				let result =
					T::XcmReserveTransfer::reserve_transfer(0, &recipient, amount, destination);
				if let Err(err) = result {
					log::error!(
						"Failed to execute xcm transfer to parachain {} - {:?}.",
						destination.para_id,
						err
					);
				}
			}
			Ok(())
		}
	}
}
