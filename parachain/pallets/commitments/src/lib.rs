#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error,
	weights::Weight,
	dispatch::DispatchResult,
};
use sp_io::offchain_index;
use sp_core::{H160, H256, RuntimeDebug};
use sp_runtime::{
	traits::{Hash, Zero},
	DigestItem
};
use codec::{Encode, Decode};
use artemis_core::{ChannelId, MessageCommitment};
use ethabi::{self, Token};
use enum_iterator::IntoEnumIterator;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Custom DigestItem for header digest
#[derive(Encode, Decode, Copy, Clone, PartialEq, RuntimeDebug)]
enum AuxiliaryDigestItem {
	Commitment(ChannelId, H256)
}

impl<T> Into<DigestItem<T>> for AuxiliaryDigestItem {
    fn into(self) -> DigestItem<T> {
        DigestItem::Other(self.encode())
    }
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
struct Message {
	target: H160,
	nonce: u64,
	payload: Vec<u8>,
}

pub trait Config: frame_system::Config {

	const INDEXING_PREFIX: &'static [u8];

	type Hashing: Hash<Output = H256>;

	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
	trait Store for Module<T: Config> as Commitments {
		/// Interval between committing messages
		Interval get(fn interval) config(): T::BlockNumber;

		/// Messages waiting to be committed
		pub MessageQueues: map hasher(identity) ChannelId => Vec<Message>;
	}
}

decl_event! {
	pub enum Event {

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
			if (now % Self::interval()).is_zero() {
				Self::commit()
			} else {
				0
			}
		}
	}
}

impl<T: Config> Module<T> {

	fn offchain_key(channel_id: ChannelId, hash: H256) -> Vec<u8> {
		(T::INDEXING_PREFIX, channel_id, hash).encode()
	}

	// TODO: return proper weight
	fn commit() -> Weight {
		let mut weight: Weight = 0;

		for channel_id in ChannelId::into_enum_iter() {
			weight += Self::commit_for_channel(channel_id);
		}

		weight
	}

	fn commit_for_channel(channel_id: ChannelId) -> Weight {
		let messages: Vec<Message> = <Self as Store>::MessageQueues::take(channel_id);
		if messages.len() == 0 {
			return 0
		}

		let commitment = Self::encode_commitment(&messages);
		let commitment_hash = <T as Config>::Hashing::hash(&commitment);

		let digest_item = AuxiliaryDigestItem::Commitment(channel_id, commitment_hash.clone()).into();
		<frame_system::Module<T>>::deposit_log(digest_item);

		offchain_index::set(&Self::offchain_key(channel_id, commitment_hash), &messages.encode());

		0
	}

	fn encode_commitment(commitment: &[Message]) -> Vec<u8> {
		let messages: Vec<Token> = commitment.iter()
			.map(|message|
				Token::Tuple(vec![
					Token::Address(message.target),
					Token::Bytes(message.payload.clone()),
					Token::Uint(message.nonce.into())
				])
			)
			.collect();

		ethabi::encode(&vec![Token::FixedArray(messages)])
	}
}

impl<T: Config> MessageCommitment for Module<T> {

	// Add a message for eventual inclusion in a commitment
	// TODO (Security): Limit number of messages per commitment
	//   https://github.com/Snowfork/polkadot-ethereum/issues/226
	fn add(channel_id: ChannelId, target: H160, nonce: u64, payload: &[u8]) -> DispatchResult {
		MessageQueues::append(
			channel_id,
			Message {
				target,
				nonce,
				payload: payload.to_vec()
			}
		);
		Ok(())
	}
}
