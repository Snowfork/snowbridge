#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::{DispatchResult, Dispatchable}};
use frame_support::{Parameter, weights::GetDispatchInfo,  dispatch::{PostDispatchInfo, DispatchError}};
use frame_system::{self as system, RawOrigin};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use pallet_broker as broker;

use artemis_core::{AppID, Message, Verifier, VerificationInput, VerifiedMessage};

pub trait Trait: system::Trait + broker::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	type Call: Parameter + Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo> + GetDispatchInfo + From<broker::Call<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VerifierModule {
		pub VerifiedPayloads: map hasher(blake2_128_concat) T::Hash => ();
	}
}

decl_event!(
	pub enum Event {

	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {

	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {
	// No-op verifier that sends verified message back to broker.
	fn do_verify(app_id: AppID, message: Message) -> DispatchResult {

		let (tx_hash, block_number) = match message.verification {
			VerificationInput::Basic { tx_hash, block_number } => (tx_hash, block_number),
		};

		let key_input = (app_id, message.payload.clone(), tx_hash.as_fixed_bytes(), block_number);
		//let key_input_bytes = key_input.encode();
		//let key = H256::from_slice(&blake2_256(&key_input_bytes[..]));

		let key = T::Hashing::hash_of(&key_input);

		if <VerifiedPayloads<T>>::contains_key(key) {
			return Err(DispatchError::Other("Message failed verification"))
		} else {
			<VerifiedPayloads<T>>::insert(key, ());
		}

		let verified_message = VerifiedMessage { payload: message.payload };
		let call: Box<<T as Trait>::Call> = Box::new(broker::Call::accept(app_id, verified_message).into());
		call.dispatch(RawOrigin::Root.into()).map(|_| ()).map_err(|e| e.error)
	}
}

impl<T: Trait> Verifier<T::AccountId> for Module<T> {
	fn verify(sender: T::AccountId, app_id: AppID, message: Message) -> DispatchResult {
		Self::do_verify(app_id, message)?;
		Ok(())
	}
}
