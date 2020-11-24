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
	traits::{Zero, Hash},
	DigestItem
};

use sp_io::hashing::keccak_256;

use codec::{Encode, Decode};

use sp_std::if_std;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Custom DigestItem for header digest
#[derive(Encode, Decode, Copy, Clone)]
enum OtherDigestItem {
	/// Message commitment for an application
	Commitment {
		/// Application address
		address: H160,
		/// Commitment to a set of messages
		commitment: H256
	}
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

	type PruneInterval: Get<Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Commitments {
		Messages: map hasher(identity) H160 => Vec<Vec<u8>>;
	}
}


decl_event! {
	pub enum Event {}
}

decl_error! {
	pub enum Error for Module<T: Trait> {}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		// Generate a message commitment every `T::PruneInterval` blocks
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

	// Generate a message commitment and prune storage
	// TODO: return proper weight
	fn commit() -> Weight {
		let mut digest = <frame_system::Module<T>>::digest();

		let mut addresses: Vec<H160> = Vec::new();

		for (address, messages) in <Self as Store>::Messages::iter() {
			// cache the storage key so we can prune it later
			addresses.push(address);

			// hash the messages and add a digest item
			let commitment: H256 = keccak_256(&messages.encode()[..]).into();
			let item = OtherDigestItem::Commitment{address, commitment};
			digest.push(DigestItem::Other(item.encode()));
		}

		// prune messages
		for address in addresses {
			<Self as Store>::Messages::remove(address)
		}

		0
	}
}

impl<T: Trait> Commitments for Module<T> {

	// Add a message for eventual inclusion in a commitment
	fn add(address: H160, payload: &[u8]) {
		let value: Vec<u8> = payload.into();
		<Self as Store>::Messages::append(address, payload);
	}
}
