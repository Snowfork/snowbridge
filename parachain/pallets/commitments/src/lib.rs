#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error,
	storage::IterableStorageMap,
	dispatch,
	weights::Weight,
	traits::Get
};
use artemis_core::Commitments;

use sp_core::{H160, H256};
use sp_runtime::{
	traits::Zero,
	DigestItem
};

use codec::{Encode, Decode};

use sp_std::if_std;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Copy, Clone)]
enum OtherDigestItem {
	Commitment(H160, H256)
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type PruneInterval: Get<Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Commitments {
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
			if (now % T::PruneInterval::get()).is_zero() {
				Self::commit()
			} else {
				0
			}
		}

	}
}

impl<T: Trait> Module<T> {

	fn commit() -> Weight {
		let mut digest = <frame_system::Module<T>>::digest();

		for (key, value) in <Self as Store>::Messages::iter() {

			if_std! {
				println!("key: {:?}", key);
				println!("value: {:?}", value);
			}
			T::Hashing::hash(value.encode())
		}



		let foo: DigestItem<T::Hash> = DigestItem::Other(Vec::from([0u8; 12]));

		digest.push(foo);

		0
	}
}

impl<T: Trait> Commitments for Module<T> {
	fn add(address: H160, payload: &[u8]) {
		let mut messages: Vec<Vec<u8>> = <Self as Store>::Messages::get(address);
		messages.push(payload.into());
		<Self as Store>::Messages::insert(address, messages);
	}
}
