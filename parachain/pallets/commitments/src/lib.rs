#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::ensure_signed;
use frame_support::storage::StorageMap;
use artemis_core::Commitments;

use sp_core::H160;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Commitments {
		CommitInterval: T::BlockNumber;
		Messages: map hasher(identity) H160 => Vec<Vec<u8>>;
	}
}


decl_event! {
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		SomethingStored(u32, AccountId),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		NoneValue,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		fn on_initialize(now: T::BlockNumber) -> Weight {
			let digest = <frame_system::Module<T>>::digest();
		}

	}
}


impl<T: Trait> Commitments for Module<T> {

	fn add(address: H160, payload: &[u8]) {
		let mut messages: Vec<Vec<u8>> = <Self as Store>::Messages::get(address);
		messages.push(payload.into());
		<Self as Store>::Messages::insert(address, messages);
	}

}
