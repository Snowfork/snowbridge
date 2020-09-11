#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system as system;
use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult};
use sp_std::prelude::*;

use artemis_core::{AppID, Message, Verifier, VerificationInput};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VerifierModule {
		RelayKey get(fn key) config(): T::AccountId;
		pub LatestBlockEvent: (u64, u32);
	}
}

decl_event!(
	pub enum Event {

	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NotSupported,
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

	fn do_verify(sender: T::AccountId, app_id: AppID, message: &Message) -> DispatchResult {
		Self::verify_sender(sender)?;

		let (block_no, event_idx) = match message.verification {
			VerificationInput::Basic { block_number, event_index } => (block_number, event_index),
			VerificationInput::None => return Err(Error::<T>::NotSupported.into())
		};

		let (latest_block_no, latest_event_idx) = <LatestBlockEvent>::get();

		if block_no < latest_block_no {
			return Err(Error::<T>::Invalid.into())
		}

		if event_idx < latest_event_idx {
			return Err(Error::<T>::Invalid.into())
		}

		<LatestBlockEvent>::set((block_no, event_idx));

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
	fn verify(sender: T::AccountId, app_id: AppID, message: &Message) -> DispatchResult {
		Self::do_verify(sender, app_id, message)?;
		Ok(())
	}
}
