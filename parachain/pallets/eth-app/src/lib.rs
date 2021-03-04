//! # ETH
//!
//! An application that implements a bridged ETH asset.
//!
//! ## Overview
//!
//! ETH balances are stored in the tightly-coupled [`asset`] runtime module. When an account holder burns
//! some of their balance, a `Transfer` event is emitted. An external relayer will listen for this event
//! and relay it to the other chain.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `burn`: Burn an ETH balance.
//!
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	traits::EnsureOrigin,
	transactional,
};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;
use sp_core::{H160, U256};

use artemis_core::{ChannelId, SingleAsset, OutboundRouter};

mod payload;
use payload::OutboundPayload;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	type Asset: SingleAsset<<Self as system::Config>::AccountId>;

	type OutboundRouter: OutboundRouter;

	type CallOrigin: EnsureOrigin<Self::Origin, Success=H160>;
}

decl_storage! {
	trait Store for Module<T: Config> as EthModule {
		/// Address of the peer application on the Ethereum side.
		Address get(fn address) config(): H160;
	}
}

decl_event!(
    /// Events for the ETH module.
	pub enum Event<T>
	where
		AccountId = <T as system::Config>::AccountId
	{
		Burned(AccountId, H160, U256),
		Minted(H160, AccountId, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		/// The submitted payload could not be decoded.
		InvalidPayload,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		// Users should burn their holdings to release funds on the Ethereum side
		// TODO: Calculate weights
		#[weight = 0]
		#[transactional]
		pub fn burn(origin, channel_id: ChannelId, recipient: H160, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Asset::withdraw(&who, amount)?;

			let message = OutboundPayload {
				sender: who.clone(),
				recipient: recipient.clone(),
				amount: amount
			};

			T::OutboundRouter::submit(channel_id, Address::get(), &message.encode())?;
			Self::deposit_event(RawEvent::Burned(who.clone(), recipient, amount));

			Ok(())
		}

		#[weight = 0]
		#[transactional]
		pub fn mint(origin, sender: H160, recipient: <T::Lookup as StaticLookup>::Source, amount: U256) -> DispatchResult {
			let who = T::CallOrigin::ensure_origin(origin)?;
			if who != Address::get() {
				return Err(DispatchError::BadOrigin.into());
			}

			let recipient = T::Lookup::lookup(recipient)?;
			T::Asset::deposit(&recipient, amount)?;
			Self::deposit_event(RawEvent::Minted(sender, recipient.clone(), amount));

			Ok(())
		}
	}
}
