#![cfg_attr(not(feature = "std"), no_std)]
use itertools::Itertools;

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
	// Generate a key for offchain storage
	fn offchain_key(channel_id: ChannelId, hash: H256) -> Vec<u8> {
		(T::INDEXING_PREFIX, channel_id, hash).encode()
	}

	// TODO: return proper weight
	fn commit() -> Weight {
		let mut weight = Self::commit_for_basic_channel();
		weight + Self::commit_for_incentivized_channel()
	}

	fn commit_for_basic_channel() -> Weight {
		let all_messages: Vec<(T::AccountId, Message)> = <Self as Store>::BasicMessageQueue::get();
		if all_messages.len() == 0 {
			return 0;
		}

		//
		// TODO: benchmark this grouping approach vs a sorting in-place solution
		//
		let subcommitments = all_messages
			.into_iter()
			.map(|(acc, v)| (acc.encode(), v))
			.into_group_map_by(|x| x.0.clone())
			.into_iter()
			.map(|(_, msgs)| {
				let msgs = msgs
					.into_iter()
					.map(|(_, msg)| msg)
					.collect::<Vec<Message>>();
				Self::encode_commitment(&msgs[..])
			});

		let subcom_hashes = subcommitments
			.into_iter()
			.map(|c| <T as Config>::Hashing::hash(&c));

		// 1. Generate Merkle Tree (stripped down generate_merkle_proof)

		// 2. Deposit log with Merkle Tree Root

		// 3. Store the messages and/or the merkle proofs off-chain

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
			&Self::offchain_key(ChannelId::Incentivized, commitment_hash),
			&messages.encode(),
		);

		0
	}

	// ABI-encode the commitment
	fn encode_commitment(commitment: &[Message]) -> Vec<u8> {
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
	fn add_basic(
		_account_id: T::AccountId,
		_target: H160,
		_nonce: u64,
		_payload: &[u8],
	) -> DispatchResult {
		Ok(())
	}
}

impl<T: Config> IncentivizedMessageCommitment for Module<T> {
	// Add a message for eventual inclusion in a commitment
	// TODO (Security): Limit number of messages per commitment
	//   https://github.com/Snowfork/polkadot-ethereum/issues/226
	fn add_incentivized(target: H160, nonce: u64, payload: &[u8]) -> DispatchResult {
		// TODO

		// MessageQueues::append(
		//     channel_id,
		//     Message {
		//         target,
		//         nonce,
		//         payload: payload.to_vec(),
		//     },
		// );
		Ok(())
	}
}
