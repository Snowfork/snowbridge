//! # Basic Channel
//!
//! The Basic Channel module is is a non-incentivized bridge between Ethereum and Pokadot
//! ecosystems. It is meant to be a low-barrier entry to cross-chain communication without guarantees,
//! but also without fees.
//!
//! ## Implementation
//!
//! Before a [`Message`] is dispatched to a target [`Application`], it is submitted to a [`Verifier`] for verification.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `submit`: Submit a message for verification and dispatch.
//!


#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
	storage::StorageMap,
};
use frame_system::{self as system, ensure_signed};

use sp_core::H160;
use sp_std::prelude::*;
use sp_std::convert::TryFrom;
use artemis_core::{
	ChannelId, SubmitOutbound, Message, MessageId,
	MessageCommitment, MessageDispatch, Verifier,
	SourceChannelConfig,
};
use channel::inbound::make_inbound_channel;
use channel::outbound::make_outbound_channel;
use primitives::{InboundChannelData, OutboundChannelData};
use envelope::Envelope;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod channel;
mod primitives;
mod envelope;

type MessageNonce = u64;

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Verifier module for message verification.
	type Verifier: Verifier;

	/// Used by outbound channels to persist messages for outbound delivery.
	type MessageCommitment: MessageCommitment;

	/// Verifier module for message verification.
	type MessageDispatch: MessageDispatch<MessageId>;
}

// The pallet's runtime storage items.
decl_storage! {
	trait Store for Module<T: Config> as BasicChannelModule {
		/// Outbound (source) channels on Ethereum from whom we will accept messages.
		pub SourceChannels: map hasher(identity) H160 => Option<ChannelId>;
		/// Storage for inbound channels.
		pub InboundChannels: map hasher(identity) ChannelId => InboundChannelData;
		/// Storage for outbound channels.
		pub OutboundChannels: map hasher(identity) ChannelId => OutboundChannelData;
	}
	add_extra_genesis {
		config(source_channels): SourceChannelConfig;
		build(|config: &GenesisConfig| {
			let sources = config.source_channels;
			SourceChannels::insert(sources.basic.address, ChannelId::Basic);
			SourceChannels::insert(sources.incentivized.address, ChannelId::Incentivized);
		});
	}
}

// Pallets use events to inform users when important changes are made.
decl_event!(
	pub enum Event {
		/// Message has been accepted by an outbound channel
		MessageAccepted(ChannelId, MessageNonce),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Message came from an invalid outbound channel on the Ethereum side.
		InvalidSourceChannel,
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
		BadNonce,
		/// Target application not found.
		AppNotFound,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn submit(origin, message: Message) -> DispatchResult {
			let relayer = ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted to us from a known
			// outbound channel on the ethereum side
			let channel_id = SourceChannels::get(envelope.channel)
				.ok_or(Error::<T>::InvalidSourceChannel)?;

			// Submit to an inbound channel for further processing
			let channel = make_inbound_channel::<T>(channel_id);
			channel.submit(&relayer, &envelope)
		}
	}
}

impl<T: Config> SubmitOutbound for Module<T> {

	// Submit a message to to Ethereum, taking into account the desired
	// channel for delivery.
	fn submit(channel_id: ChannelId, target: H160, payload: &[u8]) -> DispatchResult {
		// Construct channel object from storage
		let channel = make_outbound_channel::<T>(channel_id);
		channel.submit(target, payload)
	}
}
