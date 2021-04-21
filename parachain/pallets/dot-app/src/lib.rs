#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod payload;
pub mod weights;
pub mod primitives;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	transactional,
	traits::{
		Get,
		EnsureOrigin,
		Currency,
		ExistenceRequirement::{KeepAlive, AllowDeath},
	},
};

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;

use sp_std::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::{
	ModuleId,
	traits::{StaticLookup, AccountIdConversion},
};
use artemis_core::{ChannelId, OutboundRouter};

use primitives::{wrap, unwrap};
use payload::OutboundPayload;
pub use weights::WeightInfo;

pub use pallet::*;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

		type Currency: Currency<Self::AccountId>;

		type OutboundRouter: OutboundRouter<Self::AccountId>;

		type CallOrigin: EnsureOrigin<Self::Origin, Success=H160>;

		type ModuleId: Get<ModuleId>;

		#[pallet::constant]
		type Decimals: Get<u32>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// Verify that `T::Decimals` is 10 (DOT), or 12 (KSM) to guarantee
		// safe conversions between native and wrapped DOT.
		#[cfg(feature = "std")]
		fn integrity_test() {
			sp_io::TestExternalities::new_empty().execute_with(|| {
				let allowed_decimals: &[u32] = &[10, 12];
				let decimals = T::Decimals::get();
				assert!(
					allowed_decimals.contains(&decimals)
				)
			});
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	pub enum Event<T: Config>
	{
		Locked(T::AccountId, H160, BalanceOf<T>),
		Unlocked(H160, T::AccountId, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn address)]
	pub(super) type Address<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Illegal conversion between native and wrapped DOT.
		///
		/// In practice, this error should never occur under the conditions
		/// we've tested. If however the bridge or the peer Ethereum contract
		/// is exploited, then all bets are off.
		Overflow
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T> {
		pub address: H160,
		pub phantom: sp_std::marker::PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				address: Default::default(),
				phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Address<T>>::put(self.address);
		 }
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::lock())]
		#[transactional]
		pub fn lock(origin: OriginFor<T>, channel_id: ChannelId, recipient: H160, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Currency::transfer(&who, &Self::account_id(), amount, AllowDeath)?;

			let amount_wrapped = wrap::<T>(amount, T::Decimals::get()).ok_or(Error::<T>::Overflow)?;

			let message = OutboundPayload {
				sender: who.clone(),
				recipient: recipient.clone(),
				amount: amount_wrapped,
			};

			T::OutboundRouter::submit(channel_id, &who, <Address<T>>::get(), &message.encode())?;
			Self::deposit_event(Event::Locked(who.clone(), recipient, amount));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::unlock())]
		#[transactional]
		pub fn unlock(origin: OriginFor<T>, sender: H160, recipient: <T::Lookup as StaticLookup>::Source, amount: U256) -> DispatchResult {
			let who = T::CallOrigin::ensure_origin(origin)?;
			if who != <Address<T>>::get() {
				return Err(DispatchError::BadOrigin.into());
			}

			let amount_unwrapped = unwrap::<T>(amount, T::Decimals::get()).ok_or(Error::<T>::Overflow)?;

			let recipient = T::Lookup::lookup(recipient)?;
			T::Currency::transfer(&Self::account_id(), &recipient, amount_unwrapped, KeepAlive)?;
			Self::deposit_event(Event::Unlocked(sender, recipient, amount_unwrapped));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::ModuleId::get().into_account()
		}
	}
}
