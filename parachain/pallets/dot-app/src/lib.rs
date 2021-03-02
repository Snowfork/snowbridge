#![cfg_attr(not(feature = "std"), no_std)]
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	transactional,
	traits::{
		Get,
		EnsureOrigin,
		Currency,
		ExistenceRequirement::{KeepAlive, AllowDeath},
	}
};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;
use sp_core::H160;
use sp_runtime::{
	ModuleId,
	traits::AccountIdConversion,
	SaturatedConversion,
};

use artemis_core::{ChannelId, SubmitOutbound};

mod payload;
use payload::OutboundPayload;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	type Currency: Currency<Self::AccountId>;

	type SubmitOutbound: SubmitOutbound;

	type CallOrigin: EnsureOrigin<Self::Origin, Success=H160>;

	type ModuleId: Get<ModuleId>;
}

decl_storage! {
	trait Store for Module<T: Config> as DotModule {
		/// Address of the peer application on the Ethereum side.
		Address get(fn address) config(): H160;
	}
}

decl_event!(
    /// Events for the ETH module.
	pub enum Event<T>
	where
		AccountId = <T as system::Config>::AccountId,
		Balance = BalanceOf<T>
	{
		Locked(AccountId, H160, Balance),
		Unlocked(H160, AccountId, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {

	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		#[transactional]
		pub fn lock(origin, channel_id: ChannelId, recipient: H160, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Currency::transfer(&who, &Self::account_id(), amount, AllowDeath)?;

			let message = OutboundPayload {
				sender: who.clone(),
				recipient: recipient.clone(),
				amount: amount.saturated_into::<u128>(),
			};

			T::SubmitOutbound::submit(channel_id, Address::get(), &message.encode())?;
			Self::deposit_event(RawEvent::Locked(who.clone(), recipient, amount));
			Ok(())
		}

		#[weight = 0]
		#[transactional]
		pub fn unlock(origin, sender: H160, recipient: <T::Lookup as StaticLookup>::Source, amount: BalanceOf<T>) -> DispatchResult {
			let who = T::CallOrigin::ensure_origin(origin)?;
			if who != Address::get() {
				return Err(DispatchError::BadOrigin.into());
			}

			let recipient = T::Lookup::lookup(recipient)?;
			T::Currency::transfer(&Self::account_id(), &recipient, amount, KeepAlive)?;
			Self::deposit_event(RawEvent::Unlocked(sender, recipient, amount));
			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}
}
