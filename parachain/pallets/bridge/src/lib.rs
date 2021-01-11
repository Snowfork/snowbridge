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

use artemis_core::{AppId, Application, Message, Messages, Verifier};

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// The verifier module responsible for verifying submitted messages.
	type Verifier: Verifier<<Self as system::Config>::AccountId>;

	/// ETH Application
	type AppETH: Application;

	/// ERC20 Application
	type AppERC20: Application;
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeModule {

	}
}

decl_event!(
    /// Events for the Bridge module.
	pub enum Event {
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Target application not found. No messages were processed
		AppNotFound,
		/// Some appplication messages failed in their handler and were skipped.
		SkippedFailedMessages,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Submit `message` for dispatch to a target application identified by `app_id`.
		#[weight = 0]
		pub fn submit(origin, app_id: AppId, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// TODO: move replay protection here

			T::Verifier::verify(who, app_id, &message)?;
			Self::dispatch(app_id.into(), &message)
		}

		/// Submit multiple messages for dispatch to multiple target applications.
		#[weight = 0]
		pub fn submit_bulk(origin, messages_by_app: Vec<Messages>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Verifier::verify_bulk(who, messages_by_app.as_slice())?;

			let handlers: Vec<fn(&[u8]) -> DispatchResult> = messages_by_app.iter()
				.filter_map(|messages| Self::get_handler(messages.0.into()))
				.collect();
			if handlers.len() < messages_by_app.len() {
				return Err(Error::<T>::AppNotFound.into());
			}

			let errors: Vec<DispatchError> = messages_by_app.iter()
				.enumerate()
				.map(|(i, messages)| {
					let handler = handlers[i];
					messages.1.iter()
						.map(|msg| handler(&msg.payload))
						.collect::<Vec<DispatchResult>>()
				})
				.flatten()
				.filter_map(|r| r.err())
				.collect();

			if errors.is_empty() { Ok(()) } else { Err(Error::<T>::SkippedFailedMessages.into()) }
		}
	}
}

impl<T: Config> Module<T> {
	fn dispatch(address: H160, message: &Message) -> DispatchResult {
		let handler = Self::get_handler(address)
			.ok_or(Error::<T>::AppNotFound)?;
		handler(message.payload.as_ref())
	}

	fn get_handler(address: H160) -> Option<fn(&[u8]) -> DispatchResult> {
		if address == T::AppETH::address() {
			return Some(T::AppETH::handle);
		} else if address == T::AppERC20::address() {
			return Some(T::AppERC20::handle);
		}
		None
	}
}
