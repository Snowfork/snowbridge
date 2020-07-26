#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::{DispatchResult, Dispatchable}};
use frame_support::{Parameter, traits::schedule::Anon as ScheduleAnon};
use frame_system::{self as system, ensure_signed, ensure_root};

use sp_std::prelude::*;

use pallet_broker::{self as broker};

use common::{AppID, Message, Verifier};


pub trait Trait: system::Trait + broker::Trait {

	type Event: From<Event> + Into<<Self as system::Trait>::Event>;

	type Proposal: Parameter + Dispatchable<Origin=Self::Origin> + From<broker::Call<Self>>;	
	type Scheduler: ScheduleAnon<Self::BlockNumber, Self::Proposal>;
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

		//let delay: <T as system::Trait>::BlockNumber = 1.into();

		let delay: u32 = 1;

		if T::Scheduler::schedule(
			<system::Module<T>>::block_number() + delay.into(),
			None,
			64,
			broker::Call::accept(app_id, message).into()
		).is_err() {
			frame_support::print("ERROR: scheduling failed");
		}

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
