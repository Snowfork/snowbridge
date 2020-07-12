#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed, ensure_root};

use common::{AppID, Message, Verifier, Broker, Application};

pub trait Trait: system::Trait {

	type Event: From<Event> + Into<<Self as system::Trait>::Event>;

	type DummyVerifier: Verifier;
	type DummyApp1: Application;
	type DummyApp2: Application;

}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {

	}
}

decl_event!(
	pub enum Event {
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		HandlerNotFound
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		// Submit message to broker for processing
		//
		// The message will be routed to a verifier
		#[weight = 0]
		pub fn submit(origin, app_id: AppID, message: Message) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			T::DummyVerifier::verify(app_id, message)?;
			Ok(())
		}

		// Accept a message that has been previously verified
		//
		// Called by a verifier
		#[weight = 0]
		pub fn accept(origin, app_id: AppID, message: Message) -> DispatchResult {
			ensure_root(origin)?;
			Self::dispatch(app_id, message)?;
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	// Dispatch verified message to a target application
	fn dispatch(app_id: AppID, message: Message) -> DispatchResult {

		match app_id {
			_ if T::DummyApp1::is_handler_for(app_id) => {
				T::DummyApp1::handle(app_id, message)?;
			}
			_ if T::DummyApp2::is_handler_for(app_id) => {
				T::DummyApp2::handle(app_id, message)?;
			}
			_ => return Err(<Error<T>>::HandlerNotFound.into())
		}

		Ok(())
	}
}


impl<T: Trait> Broker for Module<T> {

	fn submit(app_id: AppID, message: Message) -> DispatchResult {
		T::DummyVerifier::verify(app_id, message)?;
		Ok(())
	}

}
