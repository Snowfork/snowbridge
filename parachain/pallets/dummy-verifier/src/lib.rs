#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::{DispatchResult, Dispatchable}};
use frame_support::{Parameter, weights::GetDispatchInfo,  dispatch::PostDispatchInfo};
use frame_system::{self as system, RawOrigin};

use sp_std::prelude::*;

use pallet_broker::{self as broker};

use artemis_core::{AppID, Message, Verifier};


pub trait Trait: system::Trait + broker::Trait {

	type Event: From<Event> + Into<<Self as system::Trait>::Event>;

	type Call: Parameter + Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo> + GetDispatchInfo + From<broker::Call<Self>>;
}

decl_storage! {

	trait Store for Module<T: Trait> as VerifierModule {

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
	fn schedule_approval(app_id: AppID, message: Message) -> DispatchResult {

		let call: Box<<T as Trait>::Call> = Box::new(broker::Call::accept(app_id, message).into());

		// we purposely swallow the error here
		let _ = call.dispatch(RawOrigin::Root.into());

		Ok(())
	}

}

impl<T: Trait> Verifier for Module<T> {

	// verify a message
	fn verify(app_id: AppID, message: Message) -> DispatchResult {

		Self::schedule_approval(app_id, message)?;

		Ok(())
	}

}
