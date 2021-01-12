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
//! This application implements the [`Application`] trait and conforms to its interface.
//!
//! ### Dispatchable Calls
//!
//! - `burn`: Burn an ERC20 token balance.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_std::convert::TryInto;
use sp_core::{H160, U256};
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
};

use artemis_core::{Application, AssetId, MultiAsset, Commitments};
use artemis_ethereum::Log;

mod payload;
use payload::{InPayload, OutPayload};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	type Assets: MultiAsset<<Self as system::Config>::AccountId>;

	type Commitments: Commitments;
}

decl_storage! {
	trait Store for Module<T: Config> as Erc20Module {
		Address get(fn address) config(): H160;
	}
}

decl_event! {
    /// Events for the ERC20 module.
	pub enum Event<T>
	where
		AccountId = <T as system::Config>::AccountId,
	{
		Burned(H160, AccountId, U256),
		Minted(H160, AccountId, U256),
		// TODO: Remove once relayer is updated to read commitments instead
		Transfer(H160, AccountId, H160, U256),
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
		#[weight = 0]
		pub fn burn(origin, token_addr: H160, recipient: H160, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Assets::withdraw(AssetId::Token(token_addr), &who, amount)?;

			let message = OutPayload {
				token_addr: token_addr,
				sender_addr: who.clone(),
				recipient_addr: recipient,
				amount: amount
			};
			T::Commitments::add(Self::address(), message.encode());

			Self::deposit_event(RawEvent::Burned(token_addr, who.clone(), amount));

			// TODO: Remove once relayer can read message commitments
			Self::deposit_event(RawEvent::Transfer(token_addr, who.clone(), recipient, amount));

			Ok(())
		}

	}
}

impl<T: Config> Module<T> {

	fn handle_event(payload: InPayload<T::AccountId>) -> DispatchResult {
		T::Assets::deposit(
			AssetId::Token(payload.token_addr),
			&payload.recipient_addr,
			payload.amount
		)?;
		Self::deposit_event(
			RawEvent::Minted(
				payload.token_addr,
				payload.recipient_addr.clone(),
				payload.amount
		));
		Ok(())
	}

}

impl<T: Config> Application for Module<T> {
	fn handle(payload: &[u8]) -> DispatchResult {
		// Decode ethereum Log event from RLP-encoded data, and try to convert to InPayload
		let payload_decoded = rlp::decode::<Log>(payload)
			.map_err(|_| Error::<T>::InvalidPayload)?
			.try_into()
			.map_err(|_| Error::<T>::InvalidPayload)?;

		Self::handle_event(payload_decoded)
	}

	fn address() -> H160 {
		Address::get()
	}
}
