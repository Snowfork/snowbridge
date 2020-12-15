//! # Verifier
//!
//! The verifier module provides functionality for message verification.
//!
//! ## Overview
//!
//! This verifier performs the following verification routines on a message:
//! - Ensuring that the message sender is trusted
//! - Ensuring that messages are not replayed
//!
//! This verifier is intended to be swapped out for an Ethereum light-client solution at some point.
//!
//! ## Interface
//!
//! The verifier implements the [`Verifier`] trait and conforms to its interface.
//!
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system as system;
use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchError, dispatch::DispatchResult};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use artemis_core::{AppId, Message, Verifier, VerificationInput, VerificationOutput};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VerifierModule {
		/// The trusted [`AccountId`] of the external relayer service.
		RelayKey get(fn key) config(): T::AccountId;

		/// Hashes of previously seen messages. Used to implement replay protection.
		pub VerifiedPayloads: map hasher(blake2_128_concat) T::Hash => ();
	}
}

decl_event!(
	pub enum Event {

	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Verification scheme is not supported.
		NotSupported,
		/// The message failed verification.
		Invalid
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {

	/// Verify a message
	fn do_verify(sender: T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		Self::verify_sender(sender)?;

		// Hash all inputs together to produce a unique key for the message
		let (block_no, event_idx) = match message.verification {
			VerificationInput::Basic { block_number, event_index } => (block_number, event_index),
			_ => return Err(Error::<T>::NotSupported.into())
		};
		let key_input = (app_id, message.payload.clone(), block_no, event_idx);
		let key = T::Hashing::hash_of(&key_input);

		// Verify that the message has not been seen before (replay protection)
		if <VerifiedPayloads<T>>::contains_key(key) {
			return Err(Error::<T>::Invalid.into())
		} else {
			<VerifiedPayloads<T>>::insert(key, ());
		}

		Ok(())
	}

	// Verify that the message sender matches the relayer account
	fn verify_sender(sender: T::AccountId) -> DispatchResult {
		if sender != RelayKey::<T>::get() {
			return Err(Error::<T>::Invalid.into())
		}
		Ok(())
	}
}

impl<T: Trait> Verifier<T::AccountId> for Module<T> {
	fn verify(sender: T::AccountId, app_id: AppId, message: &Message) -> Result<VerificationOutput, DispatchError> {
		Self::do_verify(sender, app_id, message)?;
		Ok(VerificationOutput::None)
	}
}
