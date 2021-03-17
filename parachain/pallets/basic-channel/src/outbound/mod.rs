use codec::{Decode, Encode};
use ethabi::{self, Token};
use frame_support::{decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	weights::Weight,
};
use frame_system::{self as system};
use sp_core::{RuntimeDebug, H160, H256};
use sp_io::offchain_index;
use sp_runtime::traits::{Hash, Zero};
use sp_std::prelude::*;

use artemis_core::{
	ChannelId, MessageNonce,
	types::AuxiliaryDigestItem,
};

use merkle_tree::*;

#[cfg(test)]
mod test;

pub mod merkle_tree;
pub use merkle_tree::generate_merkle_proofs;

/// Wire-format for committed messages
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub struct Message {
	/// Target application on the Ethereum side.
	pub target: H160,
	/// A nonce for replay protection and ordering.
	pub nonce: u64,
	/// Payload for target application.
	pub payload: Vec<u8>,
}

/// Wire-format for subcommitment
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub struct SubCommitment<AccountId> {
	/// The user account ID
	pub account_id: AccountId,
 	/// A list of the user messages
	pub messages: Vec<Message>,
	/// A flat commitment of the user messages
	pub flat_commitment: Vec<u8>,
}

/// Wire-format for committed Basic Channel data
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub struct CommitmentData<AccountId> {
    pub subcommitments: Vec<SubCommitment<AccountId>>
}

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Prefix for offchain storage keys.
	const INDEXING_PREFIX: &'static [u8];

	type Hashing: Hash<Output = H256>;
}

decl_storage! {
	trait Store for Module<T: Config> as BasicOutboundModule {
		/// A nonce is assigned to each origin identity
		pub Nonces: map hasher(identity) T::AccountId => u64;

		/// Interval between committing messages.
		Interval get(fn interval) config(): T::BlockNumber;

		/// Basic channel messages waiting to be committed.
		MessageQueue get(fn basic_mq): Vec<(T::AccountId, Message)>;
	}
}

decl_event! {
	pub enum Event {
		MessageAccepted(MessageNonce),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
	}
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
				Self::commit_messages()
			} else {
				0
			}
		}
	}
}

pub fn offchain_key(prefix: &[u8], hash: H256) -> Vec<u8> {
	(prefix, ChannelId::Basic, hash).encode()
}

impl<T: Config> Module<T> {
	pub fn submit(account_id: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
		Nonces::<T>::try_mutate(account_id, |nonce| -> DispatchResult {
			*nonce += 1;
			Self::push_message(account_id, target, *nonce, payload)?;
			<Module<T>>::deposit_event(Event::MessageAccepted(*nonce));
			Ok(())
		})
	}

	fn push_message(account_id: &T::AccountId, target: H160, nonce: u64, payload: &[u8]) -> DispatchResult {
		let mut mq = MessageQueue::<T>::get();
		mq.push((
			account_id.clone(),
			Message {
				target,
				nonce,
				payload: payload.to_vec(),
			},
		));
		MessageQueue::<T>::put(mq);
		Ok(())
	}

	fn commit_messages() -> Weight {
		let mut all_messages: Vec<(T::AccountId, Message)> =
			<Self as Store>::MessageQueue::get();
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
		let mut group: Option<Vec<_>> = Some(Vec::new());
		let mut prev_acc = &all_messages[0].0;
		for (acc, msg) in all_messages.iter() {
			if *acc != *prev_acc {
				let messages = group.take().unwrap();
				let flat_commitment = Self::encode_commitment(&messages);
				subcommitments.push(SubCommitment{
					account_id: acc.clone(),
					messages,
					flat_commitment,
				});
				group = Some(Vec::new());
				prev_acc = acc;
			}
			group.as_mut().map(|v| v.push(msg.clone()));
		}
		{
			let messages = group.take().unwrap();
			let flat_commitment = Self::encode_commitment(&messages);
			//let owned_group = &group.take().unwrap();
			subcommitments.push(SubCommitment{
				account_id: prev_acc.clone(),
				messages,
				flat_commitment,
			});
		}

		let subc_enc  = subcommitments
			.iter()
			.map(|subc| {
				// Flat commitment scheme for all messages of the same user
				let hash = <T as Config>::Hashing::hash(&subc.flat_commitment);
				(subc.account_id.clone(), Encode::encode(&hash))
                        })
			.collect::<Vec<(T::AccountId, Vec<u8>)>>();

		// Generate Merkle Tree
		let mroot = generate_merkle_root(subc_enc.iter().cloned().map(|(_, t)| t));

		// Deposit log with Merkle Tree Root
		let digest_item = AuxiliaryDigestItem::Commitment(ChannelId::Basic, mroot).into();
		<frame_system::Module<T>>::deposit_log(digest_item);

		let data = CommitmentData::<T::AccountId> {
			subcommitments,
		};

		let key = offchain_key(T::INDEXING_PREFIX, mroot);
		offchain_index::set(&*key, &data.encode());

		// Clear queue
		<Self as Store>::MessageQueue::put(<Vec<(T::AccountId, Message)>>::new());

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
