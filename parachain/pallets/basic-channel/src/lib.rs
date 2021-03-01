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

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};

use artemis_core::{ChannelId, Message, MessageCommitment, MessageDispatch, MessageId, Verifier};
use envelope::Envelope;
use sp_core::H160;
use sp_std::convert::TryFrom;
use sp_std::prelude::*;

mod envelope;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type MessageNonce = u64;

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	/// Verifier module for message verification.
	type Verifier: Verifier;

	/// Used by outbound channels to persist messages for outbound delivery.
	type MessageCommitment: MessageCommitment;

	/// Verifier module for message verification.
	type MessageDispatch: MessageDispatch<MessageId>;
}

decl_storage! {
	trait Store for Module<T: Config> as BasicChannelModule {
		/// Outbound (source) channel on Ethereum from whom we will accept messages.
		SourceChannel get(fn source_channel) config(): H160;
		/// Storage for inbound channels.
		pub InboundChannels: map hasher(identity) H160 => u64;
		/// Storage for outbound channels.
		pub OutboundChannels: map hasher(identity) T::AccountId => u64;
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

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn submit(origin, message: Message) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let _relayer = ensure_signed(origin)?;

			// Submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted to us from a known
			// outbound channel on the ethereum side
			ensure!(envelope.channel == SourceChannel::get(), Error::<T>::InvalidSourceChannel);

			Self::submit_inbound(&envelope)
		}
	}
}

impl<T: Config> Module<T> {
	fn submit_inbound(envelope: &Envelope) -> DispatchResult {
		InboundChannels::try_mutate(envelope.origin, |nonce| {
			if envelope.nonce != *nonce + 1 {
				return Err(Error::<T>::BadNonce);
			}
			*nonce += 1;

			Ok(())
		})?;

		let message_id = MessageId::new(ChannelId::Basic, envelope.origin, envelope.nonce);
		T::MessageDispatch::dispatch(envelope.source, message_id, &envelope.payload);

		Ok(())
	}

	// Submit a message to Ethereum
	#[allow(dead_code)]
	fn submit_outbound(account_id: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
		OutboundChannels::<T>::try_mutate(account_id, |nonce| {
			*nonce += 1;
			T::MessageCommitment::add(ChannelId::Basic, target, *nonce, payload)?;
			Self::deposit_event(Event::<T>::MessageAccepted(account_id.clone(), *nonce));
			Ok(())
		})
	}
}
