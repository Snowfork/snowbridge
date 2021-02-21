//! # Basic Channel
//!
//! The Basic Channel module is is a non-incentivized bridge between Ethereum and Pokadot
//! ecosystems. It is meant to be a low-barrier entry to cross-chain communication without
//! guarantees, but also without fees.
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

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};

use artemis_core::{
	Message, MessageCommitment, MessageDispatch, MessageId, SubmitOutbound, Verifier,
};
use channel::{inbound::BasicInboundChannel, outbound::BasicOutboundChannel};
use envelope::Envelope;
use primitives::{InboundChannel, InboundChannelData, OutboundChannel, OutboundChannelData};
use sp_core::H160 as EthAddress;
use sp_std::convert::TryFrom;
use sp_std::prelude::*;

mod channel;
mod envelope;
#[cfg(test)]
mod mock;
mod primitives;
#[cfg(test)]
mod tests;

type MessageNonce = u64;

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	/// Verifier module for message verification.
	type Verifier: Verifier;

	/// Used by outbound channels to persist messages for outbound delivery.
	type MessageCommitment: MessageCommitment<Self::AccountId>;

	/// Verifier module for message verification.
	type MessageDispatch: MessageDispatch<MessageId>;
}

decl_storage! {
	trait Store for Module<T: Config> as BasicChannelModule {
		/// Storage for inbound channels.
		pub InboundChannels: map hasher(identity) EthAddress => InboundChannelData;
		/// Storage for outbound channels.
		pub OutboundChannels: map hasher(identity) T::AccountId => OutboundChannelData;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Config>::AccountId,
	{
		/// Message has been accepted by an outbound channel
		MessageAccepted(AccountId, MessageNonce),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
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
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			let channel = BasicInboundChannel::<T>::new(envelope.source);
			channel.submit(&relayer, &envelope)
		}
	}
}

impl<T: Config> SubmitOutbound<T::AccountId> for Module<T> {
	// Submit a message to Ethereum, taking the desired channel for delivery.
	fn submit(account_id: T::AccountId, target: EthAddress, payload: &[u8]) -> DispatchResult {
		// Construct channel object from storage
		let channel = BasicOutboundChannel::<T>::new(account_id);
		channel.submit(target, payload)
	}
}
