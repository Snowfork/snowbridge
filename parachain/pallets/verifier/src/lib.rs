#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::{DispatchResult, Dispatchable}};
use frame_support::{Parameter, weights::GetDispatchInfo, dispatch::PostDispatchInfo};
use frame_system::{self as system, RawOrigin};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use pallet_broker as broker;

use artemis_core::{AppID, Broker, Message, Verifier, VerificationInput};

pub trait Trait: system::Trait + broker::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	type Broker: Broker<<Self as system::Trait>::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VerifierModule {
		RelayKey get(fn key) config(): T::AccountId;
		pub VerifiedPayloads: map hasher(blake2_128_concat) T::Hash => ();
	}
}

decl_event!(
	pub enum Event {

	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		InvalidVerificationInput,
		Unauthorized,
		Replayed
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

		// Hash all inputs together to produce a unique key for the message
		let (tx_hash, block_number) = match message.verification {
			VerificationInput::Basic { tx_hash, block_number } => (tx_hash, block_number),
			VerificationInput::None => return Err(Error::<T>::InvalidVerificationInput.into())
		};
		let key_input = (app_id, message.payload.clone(), tx_hash.as_fixed_bytes(), block_number);
		let key = T::Hashing::hash_of(&key_input);

		// Verify that the message has not been seen before (replay protection)
		if <VerifiedPayloads<T>>::contains_key(key) {
			return Err(Error::<T>::Replayed.into())
		} else {
			<VerifiedPayloads<T>>::insert(key, ());
		}

		Ok(())
	}

	// Verify that the message sender matches the relayer account
	fn verify_sender(sender: T::AccountId) -> DispatchResult {
		if sender != RelayKey::<T>::get() {
			return Err(Error::<T>::Unauthorized.into())
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
