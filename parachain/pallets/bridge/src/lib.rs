#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};

use common::{Message, AppID, Verifier};

pub trait Trait: system::Trait {

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Verifier: Verifier;
}

decl_storage! {

	trait Store for Module<T: Trait> as TemplateModule {
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		Foo(AccountId),
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

		#[weight = 0]
		pub fn send_message(origin, app_id: AppID, message: Message) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			let _ = T::Verifier::verify(app_id, message)?;

			Ok(())
		}

	}
}
