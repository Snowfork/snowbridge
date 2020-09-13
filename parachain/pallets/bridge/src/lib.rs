#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;

use artemis_core::{
	registry::{App, lookup_app},
	AppId, Application, Message, Verifier,
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

		/// Submit `message` for dispatch to an application identified by `app_id`.
		#[weight = 0]
		pub fn submit(origin, app_id: AppId, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let app = lookup_app(app_id).ok_or(Error::<T>::HandlerNotFound)?;
			Self::verify(who, app_id, &message)?;
			Self::dispatch(app, message)
		}

	}
}

impl<T: Trait> Module<T> {

	// verify message
	fn verify(sender: T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		T::Verifier::verify(sender, app_id, &message)
	}

	/// Dispatch `message` to application `app`.
	fn dispatch(app: App, message: Message) -> DispatchResult {
		match app {
			App::ETH => T::AppETH::handle(message),
			App::ERC20 => T::AppERC20::handle(message)
		}
	}

}

