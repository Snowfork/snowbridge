//! # Bridge
//!
//! The Bridge module is the primary interface for submitting external messages to the parachain.
//!
//! ## Implementation
//!
//! Before a [Message] is dispatched to a target [`Application`], it is submitted to a [`Verifier`] for verification.
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
	storage::StorageMap,
	debug,
};
use frame_system::{self as system, ensure_signed};
use sp_core::H160;
use sp_std::prelude::*;
use artemis_core::{
	ChannelId, SubmitOutbound, Message,
	MessageCommitment, Verifier, Application,
	SourceChannelConfig,
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
		pub SourceChannels: map hasher(identity) H160 => Option<ChannelId>;
		pub InboundChannels: map hasher(identity) ChannelId => InboundChannelData;
		pub OutboundChannels: map hasher(identity) ChannelId => OutboundChannelData;
	}
	add_extra_genesis {
		config(source_channels): SourceChannelConfig;
		build(|config: &GenesisConfig<T>| {
			let sources = config.source_channels;
			SourceChannels::insert(sources.basic.address, ChannelId::Basic);
			SourceChannels::insert(sources.incentivized.address, ChannelId::Incentivized);
		});
	}
}

decl_event! {
    /// Events for the Bridge module.
	pub enum Event {

	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Message came from an invalid source channel
		InvalidSourceChannel,

		/// Message has an unexpected nonce
		BadNonce,

		/// Target application not found.
		AppNotFound,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn submit(origin, message: Message) -> DispatchResult {
			let relayer = ensure_signed(origin)?;

			let envelope = T::Verifier::verify(&message)?;

			let channel_id = SourceChannels::get(envelope.channel)
				.ok_or(Err(Error::<T>::InvalidSourceChannel.into()))?;

			let mut channel = make_inbound_channel::<T>(channel_id);

			channel.submit(&relayer, envelope)
		}
	}
}

impl<T: Config> Module<T> {
	fn dispatch(source: H160, payload: &[u8]) -> DispatchResult {
		if source == T::AppETH::address() {
			T::AppETH::handle(payload)
		} else if source == T::AppERC20::address() {
			T::AppERC20::handle(payload)
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
