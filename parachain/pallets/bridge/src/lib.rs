//! # Bridge
//!
//! The Bridge module is the primary interface for submitting external messages to the parachain.
//!
//! ## Implementation
//!
//! Before a [Message] is dispatched to a target [`Application`], it is submitted to a [`Verifier`] for verification. The target application is determined using the [`AppId`] submitted along with the message.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `submit`: Submit a message for verification and dispatch.
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchError, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;
use sp_core::H160;

use artemis_core::{
	AppId, Application, Message, Verifier, VerificationOutput,
};

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;

	/// The verifier module responsible for verifying submitted messages.
	type Verifier: Verifier<<Self as system::Trait>::AccountId>;

	/// ETH Application
	type AppETH: Application;

	/// ERC20 Application
	type AppERC20: Application;
}

decl_storage! {
	trait Store for Module<T: Trait> as BridgeModule {

	}
}

decl_event!(
    /// Events for the Bridge module.
	pub enum Event {
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
    	/// Target application not found.
		AppNotFound
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Submit `message` for dispatch to a target application identified by `app_id`.
		#[weight = 0]
		pub fn submit(origin, app_id: AppId, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let verification_output = Self::verify(who, app_id, &message)?;
			Self::dispatch(app_id.into(), &message, &verification_output)
		}
	}
}

impl<T: Trait> Module<T> {

	fn verify(sender: T::AccountId, app_id: AppId, message: &Message) -> Result<VerificationOutput, DispatchError> {
		T::Verifier::verify(sender, app_id, &message)
	}

	fn dispatch(address: H160, message: &Message, verification_output: &VerificationOutput) -> DispatchResult {
		if address == T::AppETH::address() {
			T::AppETH::handle(message.payload.as_ref(), verification_output)
		} else if address == T::AppERC20::address() {
			T::AppERC20::handle(message.payload.as_ref(), verification_output)
		} else {
			Err(Error::<T>::AppNotFound.into())
		}
	}

}
