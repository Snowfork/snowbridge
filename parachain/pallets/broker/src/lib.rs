#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system as system;

use sp_std::prelude::*;

use artemis_core::{
	registry::{AppName, REGISTRY},
	AppID, Application, Broker, Message, Verifier,
};

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	type Verifier: Verifier<<Self as system::Trait>::AccountId>;
	type AppETH: Application;
	type AppERC20: Application;
}

decl_storage! {
	trait Store for Module<T: Trait> as BrokerModule {

	}
}

decl_event!(
	pub enum Event {}
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

	}
}

impl<T: Trait> Module<T> {

	// Dispatch verified message to a target application
	fn dispatch(app_id: AppID, message: Message) -> DispatchResult {
		for entry in REGISTRY.iter() {
			match (entry.name, entry.id) {
				(AppName::ETH, app_id) => {
					return T::AppETH::handle(message.clone());
				}
				(AppName::ERC20, app_id) => {
					return T::AppERC20::handle(message.clone());
				}
			};
		}
		Err(Error::<T>::HandlerNotFound.into())
	}
}

impl<T: Trait> Broker<T::AccountId> for Module<T> {

	// Submit message to broker for processing
	fn submit(sender: T::AccountId, app_id: AppID, message: Message) -> DispatchResult {
		T::Verifier::verify(sender, app_id, &message)?;
		Self::dispatch(app_id, message)
	}

}
