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

use std::collections::btree_map::IntoValues;

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;
use sp_core::H160;

use sp_runtime::RuntimeDebug;

use codec::{Encode, Decode}

use artemis_core::{AppId, Application, Message, Verifier};

use primitives::{self, ChannelId, MessageNonce};

pub mod primitives;
pub mod inbound_channel;

pub trait ApplicationRegistry {

	fn iter() -> Iterator<Application>;
}

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// The verifier module responsible for verifying submitted messages.
	type Verifier: Verifier<<Self as system::Config>::AccountId>;

	type ApplicationRegistry: ApplicationRegistry;
}
// Storage for inbound channels
//
// Adapted from parity-bridges-common (modules/message-lane/lib.rs)
struct RuntimeInboundChannelStorage<T: Config> {
	cached_data: RefCell<Option<InboundChannelData>>,
}

impl<T: Config> RuntimeInboundChannelStorage {
	fn data(&self) -> InboundChannelData {
		match self.cached_data.clone().into_inner() {
			Some(data) => data,
			None => {
				let data = InboundChannelData::<T>::get(&self.lane_id);
				*self.cached_data.try_borrow_mut().expect(
					"we're in the single-threaded environment;\
						we have no recursive borrows; qed",
				) = Some(data.clone());
				data
			}
		}
	}

	fn set_data(&mut self, data: InboundLaneData<T::InboundRelayer>) {
		*self.cached_data.try_borrow_mut().expect(
			"we're in the single-threaded environment;\
				we have no recursive borrows; qed",
		) = Some(data.clone());
		InboundChannelData::<T>::set(data)
	}
}

decl_storage! {
	trait Store for Module<T: Config> as BridgeModule {
		InboundChannelData: primitives::InboundChannelData;
	}
}

decl_event! {
    /// Events for the Bridge module.
	pub enum Event {

		/// Message accepted for delivery to Ethereum
		MessageAccepted(ChannelId, MessageNonce)
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

			// Construct channel object from storage
			let channel = Self::get_inbound_channel(channel_id);

			// Design choices:
			//   Do verification and then submit to channel?
			channel.submit(&message)

		}
	}
}

impl<T: Config> Module<T> {


	fn submit_to_ethereum(channel_id: ChannelId, payload: &[u8]) {
		// Construct channel object from storage
		let channel = Self::get_outbound_channel(channel_id);

		let nonce = channel.submit(payload);
		Self::deposit_event(RawEvent::MessageAccepted(channel_id, nonce));

	}

}
