#![cfg_attr(not(feature = "std"), no_std)]

use artemis_core::{BasicMessageCommitment, ChannelId, IncentivizedMessageCommitment};
use codec::{Decode, Encode};
use ethabi::{self, Token};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, weights::Weight,
};

use sp_core::{RuntimeDebug, H160, H256};
use sp_io::offchain_index;
use sp_runtime::{
	traits::{Hash, Zero},
	DigestItem,
};
use sp_std::prelude::*;

use merkle_tree::*;

mod merkle_tree;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Auxiliary [`DigestItem`] to include in header digest.
#[derive(Encode, Decode, Copy, Clone, PartialEq, RuntimeDebug)]
pub enum AuxiliaryDigestItem {
	/// A batch of messages has been committed.
	Commitment(ChannelId, H256),
}

impl<T> Into<DigestItem<T>> for AuxiliaryDigestItem {
	fn into(self) -> DigestItem<T> {
		DigestItem::Other(self.encode())
	}
}

/// Wire-format for committed messages
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub struct Message {
	/// Target application on the Ethereum side.
	target: H160,
	/// A nonce for replay protection and ordering.
	nonce: u64,
	/// Payload for target application.
	payload: Vec<u8>,
}

/// Wire-format for committed BasicChannel data
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub struct BasicChannelBlob {
	messages: Vec<Message>,
	// TODO: store proofs
}

pub trait Config: frame_system::Config {
	/// Prefix for offchain storage keys.
	const INDEXING_PREFIX: &'static [u8];

	type Hashing: Hash<Output = H256>;

	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
	trait Store for Module<T: Config> as Commitments {
		/// Interval between committing messages.
		Interval get(fn interval) config(): T::BlockNumber;

		/// Basic channel messages waiting to be committed.
		BasicMessageQueue get(fn basic_mq): Vec<(T::AccountId, Message)>;

		/// Messages waiting to be committed.
		IncentivizedMessageQueue get(fn incentivized_mq): Vec<Message>;
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

		// Generate a message commitment every [`Interval`] blocks.
		//
		// The commitment hash is included in an [`AuxiliaryDigestItem`] in the block header,
		// with the corresponding commitment is persisted offchain.
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
	// Generate a key for basic offchain storage
	fn offchain_key_basic(account_id: T::AccountId, hash: H256) -> Vec<u8> {
		(T::INDEXING_PREFIX, ChannelId::Basic, account_id, hash).encode()
	}

	// Generate a key for incentivized offchain storage
	fn offchain_key_incentivized(hash: H256) -> Vec<u8> {
		(T::INDEXING_PREFIX, ChannelId::Incentivized, hash).encode()
	}

	// TODO: return proper weight
	fn commit() -> Weight {
		Self::commit_for_basic_channel() + Self::commit_for_incentivized_channel()
	}

	fn commit_for_basic_channel() -> Weight {
		let mut all_messages: Vec<(T::AccountId, Message)> =
			<Self as Store>::BasicMessageQueue::get();
		if all_messages.is_empty() {
			return 0;
		}

		//
		// The algorithm consists of sorting and then iterating over the contiguous
		// account messages and creating subcommitments per user message groups.
		// This algorithm is O(n log n).
		// An alternative approach would be to use a hashmap (itertool's group-by
		// won't work with Wasm). Even though it would be amortized O(n), the
		// required allocations have an impact on effective performance.
		// TODO: benchmark sorting vs a hash-map approach
		//
		all_messages.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
		let mut subcommitments = Vec::new();
		let mut messages_by_user = Vec::new();
		let mut group = Vec::new();
		let mut prev_acc = &all_messages[0].0;
		for (acc, msg) in all_messages.iter() {
			if acc != prev_acc {
				subcommitments.push((acc, Self::encode_commitment(&group)));
				messages_by_user.push((acc, group.clone()));
				group.clear();
				prev_acc = acc;
			}
			group.push(msg.clone());
		}
		subcommitments.push((prev_acc, Self::encode_commitment(&group)));
		messages_by_user.push((prev_acc, group));

		let subcom_hashes = subcommitments
			.iter()
			.map(|(_, c)| <T as Config>::Hashing::hash(&c));

		// Generate Merkle Tree
		let mt = MerkleTree::new(subcom_hashes);

		// Deposit log with Merkle Tree Root
		let digest_item = AuxiliaryDigestItem::Commitment(ChannelId::Basic, mt.root).into();
		<frame_system::Module<T>>::deposit_log(digest_item);

		// Create an off-chain storage entry per user
		messages_by_user.into_iter().for_each(|(acc, msgs)| {
			// Store the messages blob off-chain
			// TODO: store proofs?
			let blob = BasicChannelBlob { messages: msgs };
			offchain_index::set(
				&Self::offchain_key_basic((*acc).clone(), mt.root),
				&blob.encode(),
			);
		});

		0
	}

	fn commit_for_incentivized_channel() -> Weight {
		let messages: Vec<Message> = <Self as Store>::IncentivizedMessageQueue::get();
		if messages.len() == 0 {
			return 0;
		}

		let commitment = Self::encode_commitment(&messages);
		let commitment_hash = <T as Config>::Hashing::hash(&commitment);

		let digest_item =
			AuxiliaryDigestItem::Commitment(ChannelId::Incentivized, commitment_hash.clone())
				.into();
		<frame_system::Module<T>>::deposit_log(digest_item);

		offchain_index::set(
			&Self::offchain_key_incentivized(commitment_hash),
			&messages.encode(),
		);

		0
	}

	// ABI-encode the commitment
	fn encode_commitment(commitment: &Vec<Message>) -> Vec<u8> {
		let messages: Vec<Token> = commitment
			.iter()
			.map(|message| {
				Token::Tuple(vec![
					Token::Address(message.target),
					Token::Uint(message.nonce.into()),
					Token::Bytes(message.payload.clone()),
				])
			})
			.collect();
		ethabi::encode(&vec![Token::Array(messages)])
	}
}

impl<T: Config> BasicMessageCommitment<T::AccountId> for Module<T> {
	fn add(account_id: T::AccountId, target: H160, nonce: u64, payload: &[u8]) -> DispatchResult {
		let mut mq = BasicMessageQueue::<T>::get();
		mq.push((
			account_id,
			Message {
				target,
				nonce,
				payload: payload.to_vec(),
			},
		));
		BasicMessageQueue::<T>::put(mq);
		Ok(())
	}
}

impl<T: Config> IncentivizedMessageCommitment for Module<T> {
	// Add a message for eventual inclusion in a commitment
	// TODO (Security): Limit number of messages per commitment
	//   https://github.com/Snowfork/polkadot-ethereum/issues/226
	fn add(target: H160, nonce: u64, payload: &[u8]) -> DispatchResult {
		let mut mq = IncentivizedMessageQueue::get();
		mq.push(Message {
			target,
			nonce,
			payload: payload.to_vec(),
		});
		IncentivizedMessageQueue::put(mq);
		Ok(())
	}
}
