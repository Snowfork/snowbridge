#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error,
	weights::Weight,
	traits::Get
};

use sp_io::hashing::keccak_256;
use sp_core::{H160, H256, RuntimeDebug};
use sp_runtime::{
	traits::Zero,
	DigestItem
};

use codec::{Encode, Decode};
use artemis_core::Commitments;


use ethabi::{self, Token};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Custom DigestItem for header digest
#[derive(Encode, Decode, Copy, Clone, PartialEq, RuntimeDebug)]
enum CustomDigestItem {
	Commitment(H256)
}

impl<T> Into<DigestItem<T>> for CustomDigestItem {
    fn into(self) -> DigestItem<T> {
        DigestItem::Other(self.encode())
    }
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
struct Message {
	address: H160,
	payload: Vec<u8>,
	nonce: u64,
}

pub trait Config: frame_system::Config {
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

	type CommitInterval: Get<Self::BlockNumber>;
}

decl_storage! {
	trait Store for Module<T: Config> as Commitments {
		/// Nonce
		pub Nonce get(fn nonce): u64;

		/// Messages waiting to be committed
		pub MessageQueue: Vec<Message>;

		/// Committed Messages (encoded form)
		pub Commitment: Vec<u8>;
	}
}

decl_event! {
	pub enum Event {
		Commitment(H256),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		// Generate a message commitment every `T::CommitInterval` blocks.
		//
		// The hash of the commitment is stored as a digest item `CustomDigestItem::Commitment`
		// in the block header. The committed messages are persisted into storage.

		fn on_initialize(now: T::BlockNumber) -> Weight {
			if (now % T::CommitInterval::get()).is_zero() {
				Self::commit()
			} else {
				0
			}
		}
	}
}

impl<T: Config> Module<T> {

	// Generate a message commitment
	// TODO: return proper weight
	fn commit() -> Weight {
		let messages: Vec<Message> = <Self as Store>::MessageQueue::take();

		let commitment = Self::encode_commitment(&messages);
		let commitment_hash: H256 = keccak_256(&commitment).into();

		<Self as Store>::Commitment::set(commitment);

		let digest_item = CustomDigestItem::Commitment(commitment_hash.clone()).into();
		<frame_system::Module<T>>::deposit_log(digest_item);

		Self::deposit_event(Event::Commitment(commitment_hash));

		0
	}

	fn encode_commitment(commitment: &[Message]) -> Vec<u8> {
		let messages: Vec<Token> = commitment.iter()
			.map(|message|
				Token::Tuple(vec![
					Token::Address(message.address),
					Token::Bytes(message.payload.clone()),
					Token::Uint(message.nonce.into())
				])
			)
			.collect();

		ethabi::encode(&vec![Token::FixedArray(messages)])
	}

}

impl<T: Config> Commitments for Module<T> {

	// Add a message for eventual inclusion in a commitment
	// TODO: Number of messages per commitment should be bounded
	fn add(address: H160, payload: Vec<u8>) {
		let nonce = <Self as Store>::Nonce::get();
		<Self as Store>::MessageQueue::append(Message { address, payload, nonce });
		<Self as Store>::Nonce::set(nonce + 1);
	}
}
