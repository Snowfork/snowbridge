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
	dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;

use artemis_core::{ChannelId, Application, Message, Verifier};

use channel::inbound::make_inbound_channel;
use channel::outbound::make_outbound_channel;

mod channel;
pub mod primitives;


pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// The verifier module responsible for verifying submitted messages.
	type Verifier: Verifier<<Self as system::Config>::AccountId>;

	// TODO figure out how to handle applications generically
	//   For this to happen, we need to instantiate shim objects which talk to the underlying application pallets.
	type PolkaETH: Application;
	type PolkaERC20: Application;
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeModule {
		InboundChannelData: primitives::InboundChannelData;
		OutboundChannelData: primitives::OutboundChannelData;
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
		AppNotFound
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn submit(origin, channel_id: ChannelId, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut channel = make_inbound_channel::<T>(channel_id);
			channel.submit(&message)
		}
	}
}

impl<T: Config> Module<T> {

	fn submit_to_ethereum(channel_id: ChannelId, payload: &[u8]) -> DispatchResult {
		// Construct channel object from storage
		let mut channel = make_outbound_channel::<T>(channel_id);
		channel.submit(payload)
	}

}
