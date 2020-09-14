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

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;

use artemis_core::{
	registry::{App, lookup_app},
	AppId, Application, Message, Verifier,
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
			let app = lookup_app(app_id).ok_or(Error::<T>::AppNotFound)?;
			Self::verify(who, app_id, &message)?;
			Self::dispatch(app, message)
		}

	}
}

impl<T: Trait> Module<T> {

	fn verify(sender: T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		T::Verifier::verify(sender, app_id, &message)
	}

	fn dispatch(app: App, message: Message) -> DispatchResult {
		match app {
			App::ETH => T::AppETH::handle(message.payload),
			App::ERC20 => T::AppERC20::handle(message.payload)
		}
	}

}

