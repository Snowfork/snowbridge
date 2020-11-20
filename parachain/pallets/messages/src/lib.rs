#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get};
use frame_system::ensure_signed;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {

	trait Store for Module<T: Trait> as TemplateModule {

		Something get(fn something): Option<u32>;
	}
}


decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
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

		#[weight = 10]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

	}
}
