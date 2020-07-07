#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {

	trait Store for Module<T: Trait> as TemplateModule {
		Something get(fn something): Option<u32>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		SomethingStored(u32, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NoneValue,
		StorageOverflow,
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 50]
		pub fn verify_message(origin, app_id: u32, message: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			Something::put(something);

			Ok(())
		}

	}
}
