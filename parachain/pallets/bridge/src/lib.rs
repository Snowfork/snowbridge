#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;

use artemis_core::{
	registry::{AppName, REGISTRY},
	AppID, Application, Message, Verifier,
};

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	type Verifier: Verifier<<Self as system::Trait>::AccountId>;
	type AppETH: Application;
	type AppERC20: Application;
}

decl_storage! {
	trait Store for Module<T: Trait> as BridgeModule {

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

		#[weight = 0]
		pub fn submit(origin, app_id: AppID, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Verifier::verify(who, app_id, &message)?;

			Self::dispatch(app_id, message)
		}

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

