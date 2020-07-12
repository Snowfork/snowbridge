#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch::DispatchResult};
use frame_system::{self as system};

use common::{AppID, Message, Application};


pub trait Trait: system::Trait {

	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {

	trait Store for Module<T: Trait> as DummyAppModule {

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

impl<T: Trait> Application for Module<T> {

	fn is_handler_for(app_id: AppID) -> bool {
		true
	}

	fn handle(app_id: AppID, message: Message) -> DispatchResult {
		Ok(())
	}

}
