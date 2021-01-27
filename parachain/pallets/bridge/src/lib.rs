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
	dispatch::{DispatchError, DispatchResult},
	debug,
};
use frame_system::{self as system, ensure_signed};
use sp_core::H160;
use sp_std::prelude::*;
use artemis_core::{
	AppId, ChannelId, SubmitOutbound, Message,
	MessageCommitment, Verifier, Application,
};
use channel::inbound::make_inbound_channel;
use channel::outbound::make_outbound_channel;
use primitives::{InboundChannelData, OutboundChannelData};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod channel;
mod primitives;

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// The verifier module responsible for verifying submitted messages.
	type Verifier: Verifier<<Self as system::Config>::AccountId>;

	/// ETH Application
	type AppETH: Application;

	/// ERC20 Application
	type AppERC20: Application;

	type MessageCommitment: MessageCommitment;
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeModule {
		pub InboundChannels: map hasher(identity) ChannelId => InboundChannelData;
		pub OutboundChannels: map hasher(identity) ChannelId => OutboundChannelData;
	}
}

decl_event! {
    /// Events for the Bridge module.
	pub enum Event {

	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Target application not found.
		AppNotFound,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn submit(origin, channel_id: ChannelId, app_id: AppId, message: Message) -> DispatchResult {
			let relayer = ensure_signed(origin)?;

			let mut channel = make_inbound_channel::<T>(channel_id);
			channel.submit(&relayer, app_id, &message)
		}

		/// Submit multiple messages for dispatch to multiple target applications.
		#[weight = 0]
		pub fn submit_bulk(origin, messages: Vec<(AppId, Message)>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			debug::RuntimeLogger::init();

			debug::trace!(
				target: "submit_messages",
				"Received {} messages",
				messages.len(),
			);

			T::Verifier::verify_bulk(who, messages.as_slice())?;

			debug::trace!(
				target: "submit_messages",
				"Message verification succeeded",
			);

			let errors: Vec<DispatchError> = messages.iter()
				.map(|(app_id, msg)| Self::dispatch(app_id.into(), msg))
				.filter_map(|r| r.err())
				.collect();

			debug::trace!(
				target: "submit_messages",
				"Messages were dispatched",
			);

			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	fn dispatch(address: H160, message: &Message) -> DispatchResult {
		if address == T::AppETH::address() {
			T::AppETH::handle(message.payload.as_ref())
		} else if address == T::AppERC20::address() {
			T::AppERC20::handle(message.payload.as_ref())
		} else {
			Err(Error::<T>::AppNotFound.into())
		}
	}
}

impl<T: Config> SubmitOutbound for Module<T> {
	fn submit(channel_id: ChannelId, payload: &[u8]) -> DispatchResult {
		// Construct channel object from storage
		let channel = make_outbound_channel::<T>(channel_id);
		channel.submit(payload)
	}
}
