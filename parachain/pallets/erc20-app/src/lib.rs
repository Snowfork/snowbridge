//! # ERC20
//!
//! An application that implements bridged ERC20 token assets.
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
//! - `burn`: Burn an ERC20 token balance.
//!
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	traits::EnsureOrigin,
	transactional,
	weights::Weight,
};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;
use sp_core::{H160, U256};

use artemis_core::{ChannelId, OutboundRouter, AssetId, MultiAsset};

mod payload;
use payload::OutboundPayload;

mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Weight functions needed for this pallet.
pub trait WeightInfo {
	fn burn() -> Weight;
	fn mint() -> Weight;
}

impl WeightInfo for () {
	fn burn() -> Weight { 0 }
	fn mint() -> Weight { 0 }
}

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	type Assets: MultiAsset<<Self as system::Config>::AccountId>;

	type OutboundRouter: OutboundRouter<Self::AccountId>;

	type CallOrigin: EnsureOrigin<Self::Origin, Success=H160>;

	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as Erc20Module {
		/// Address of the peer application on the Ethereum side.
		Address get(fn address) config(): H160;
	}
}

decl_event! {
    /// Events for the ERC20 module.
	pub enum Event<T>
	where
		AccountId = <T as system::Config>::AccountId,
	{
		Burned(H160, AccountId, H160, U256),
		Minted(H160, H160, AccountId, U256),
	}
}

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

		/// Burn an ERC20 token balance
		#[weight = T::WeightInfo::burn()]
		#[transactional]
		pub fn burn(origin, channel_id: ChannelId, token: H160, recipient: H160, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Assets::withdraw(AssetId::Token(token), &who, amount)?;

			let message = OutboundPayload {
				token: token,
				sender: who.clone(),
				recipient: recipient.clone(),
				amount: amount
			};

			T::OutboundRouter::submit(channel_id, &who, Address::get(), &message.encode())?;
			Self::deposit_event(RawEvent::Burned(token, who.clone(), recipient, amount));

			Ok(())
		}

		#[weight = T::WeightInfo::mint()]
		#[transactional]
		pub fn mint(origin, token: H160, sender: H160, recipient: <T::Lookup as StaticLookup>::Source, amount: U256) -> DispatchResult {
			let who = T::CallOrigin::ensure_origin(origin)?;
			if who != Address::get() {
				return Err(DispatchError::BadOrigin.into());
			}

			let recipient = T::Lookup::lookup(recipient)?;
			T::Assets::deposit(AssetId::Token(token), &recipient, amount)?;
			Self::deposit_event(RawEvent::Minted(token, sender, recipient, amount));

			Ok(())
		}

	}
}
