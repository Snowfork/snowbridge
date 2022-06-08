pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use core::fmt::Debug;

use codec::{Decode, Encode, MaxEncodedLen};
use ethabi::{self, Token};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{EnsureOrigin, Get},
	BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use scale_info::TypeInfo;
use sp_core::{H160, H256};
use sp_runtime::traits::{Hash, StaticLookup, Zero};

use sp_std::prelude::*;

use snowbridge_core::{types::AuxiliaryDigestItem, ChannelId};

use std::collections::BTreeMap;

pub use weights::WeightInfo;

/// Wire-format for committed messages
#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, MaxEncodedLen, TypeInfo,
)]
#[scale_info(skip_type_params(M, N))]
// TODO: figure out why this mel_bound call is unnecessary
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct MessageBundle<AccountId, M: Get<u32>, N: Get<u32>>
where
	AccountId: Encode + Decode + Clone + PartialEq + Debug + MaxEncodedLen + TypeInfo,
{
	source_channel_id: u8,
	account: AccountId,
	/// Unique nonce for to prevent replaying bundles
	#[codec(compact)]
	nonce: u64,
	messages: BoundedVec<Message<M>, N>,
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, MaxEncodedLen, TypeInfo,
)]
#[scale_info(skip_type_params(M))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct EnqueuedMessage<AccountId, M: Get<u32>>
where
	AccountId: Encode + Decode + Clone + PartialEq + Debug + MaxEncodedLen + TypeInfo,
{
	account: AccountId,
	message: Message<M>,
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, MaxEncodedLen, TypeInfo,
)]
#[scale_info(skip_type_params(M))]
#[codec(mel_bound())]
pub struct Message<M: Get<u32>> {
	/// Unique message ID
	#[codec(compact)]
	id: u64,
	/// Target application on the Ethereum side.
	target: H160,
	/// Payload for target application.
	payload: BoundedVec<u8, M>,
}

pub type MessageBundleOf<T> = MessageBundle<
	<T as frame_system::Config>::AccountId,
	<T as Config>::MaxMessagePayloadSize,
	<T as Config>::MaxMessagesPerCommit,
>;
pub type EnqueuedMessageOf<T> =
	EnqueuedMessage<<T as frame_system::Config>::AccountId, <T as Config>::MaxMessagePayloadSize>;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Hashing: Hash<Output = H256>;

		/// Max bytes in a message payload
		#[pallet::constant]
		type MaxMessagePayloadSize: Get<u32>;

		/// Max number of messages per commitment
		#[pallet::constant]
		type MaxMessagesPerCommit: Get<u32>;

		type SetPrincipalOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MessageAccepted(u64),
		Committed { hash: H256, data: MessageBundleOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message payload exceeds byte limit.
		PayloadTooLarge,
		/// No more messages can be queued for the channel during this commit cycle.
		QueueSizeLimitReached,
		/// Cannot increment nonce
		Overflow,
		/// Not authorized to send message
		NotAuthorized,
	}

	/// Interval between commitments
	#[pallet::storage]
	#[pallet::getter(fn interval)]
	pub(super) type Interval<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Messages waiting to be committed.
	#[pallet::storage]
	pub(super) type MessageQueue<T: Config> =
		StorageValue<_, BoundedVec<EnqueuedMessageOf<T>, T::MaxMessagesPerCommit>, ValueQuery>;

	// Need a nonce for each account (message bundle) now
	#[pallet::storage]
	pub type Nonces<T: Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	pub type NextId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub interval: T::BlockNumber,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { interval: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Interval<T>>::put(self.interval);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// Generate a message commitment every [`Interval`] blocks.
		//
		// The commitment hash is included in an [`AuxiliaryDigestItem`] in the block header,
		// with the corresponding commitment is persisted offchain.
		fn on_initialize(now: T::BlockNumber) -> Weight {
			if (now % Self::interval()).is_zero() {
				Self::commit()
			} else {
				T::WeightInfo::on_initialize_non_interval()
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::set_principal())]
		pub fn set_principal(
			origin: OriginFor<T>,
			principal: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			// TODO: remove this call - might require change to runtime
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Submit message on the outbound channel
		pub fn submit(who: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
			ensure!(
				<MessageQueue<T>>::decode_len().unwrap_or(0)
					< T::MaxMessagesPerCommit::get() as usize,
				Error::<T>::QueueSizeLimitReached,
			);
			ensure!(
				payload.len() <= T::MaxMessagePayloadSize::get() as usize,
				Error::<T>::PayloadTooLarge,
			);

			let next_id = <NextId<T>>::get();
			if next_id.checked_add(1).is_none() {
				return Err(Error::<T>::Overflow.into());
			}

			<MessageQueue<T>>::try_append(EnqueuedMessage {
				account: who.clone(),
				message: Message {
					id: next_id,
					target,
					payload: payload
						.to_vec()
						.try_into()
						.map_err(|_| Error::<T>::PayloadTooLarge)?,
				},
			})
			.map_err(|_| Error::<T>::QueueSizeLimitReached)?;
			Self::deposit_event(Event::MessageAccepted(next_id));

			<NextId<T>>::put(next_id + 1);

			Ok(())
		}

		fn commit() -> Weight {
			// for every account id in the message queues map, create an Eth ABI-encoded message bundle
			// these encoded bundles will be the leaves of a merkle tree
			let message_queue = <MessageQueue<T>>::take();
			if message_queue.is_empty() {
				return T::WeightInfo::on_initialize_no_messages();
			}
			let message_count = message_queue.len();

			let average_payload_size = Self::average_payload_size(&message_queue);

			let messages_per_account: BTreeMap<T::AccountId, BoundedVec<Message<T::MaxMessagePayloadSize>, T::MaxMessagesPerCommit>> =
				message_queue.into_iter().fold(BTreeMap::new(), |mut messages_for_accounts, enqueued_message|{
					let (account, message) = (enqueued_message.account, enqueued_message.message);

					if let Some(messages) = messages_for_accounts.get_mut(&account) {
						// We should be able to safely ignore the result here, since we can't have
						// more messages for a single account than we have in the message queue
						messages.try_push(message);
					} else {
						let mut messages = BoundedVec::default();
						// Safe to ignore the result because we just created the empty BoundedVec
						#[allow(unused_must_use)]
						messages.try_push(message);
						messages_for_accounts.insert(account, messages);
					}
					messages_for_accounts
				});

			// Alternate implementation with a for loop, created while fighting the borrow-checker
			// and before discovering get_mut 🤦
			// Do we prefer the style of folding or for loops?

			// let mut messages_per_account: BTreeMap<T::AccountId, BoundedVec<Message<T::MaxMessagePayloadSize>, T::MaxMessagesPerCommit>> = BTreeMap::new();
			// for enqueued_message in message_queue {
			// 	let (account, message) = (enqueued_message.account, enqueued_message.message);

			// 	if let Some(messages) = messages_per_account.get_mut(&account) {
			// 		messages.try_push(message);
			// 	} else {
			// 		let mut messages = BoundedVec::default();
			// 		messages.try_push(message);
			// 		messages_per_account.insert(account, messages);
			// 	}
			// }

			let message_bundles_for_accounts =
				messages_per_account.into_iter().map(|(account, messages)| {
					// TODO: find a better way to handle this. We don't want to ignore all
					// messages for an account if the conversion above from Vec to BoundedVec
					// fails
					let next_nonce = <Nonces<T>>::mutate(account.clone(), |nonce| nonce.saturating_add(1));
					let bundle: MessageBundleOf<T> = MessageBundle {
						source_channel_id: ChannelId::Basic as u8,
						account,
						nonce: next_nonce,
						messages,
					};
					bundle
					});

			// TODO: create a merkle tree from these encoded bundles
			// use the merkle root as the commitment hash
			// let commitment_hash = Self::make_commitment_hash(&bundle);
			// let digest_item =
			// 	AuxiliaryDigestItem::Commitment(ChannelId::Basic, commitment_hash.clone()).into();
			// <frame_system::Pallet<T>>::deposit_log(digest_item);
			// // deposit non-ABI-encoded message bundles as events, so that the relayer can read them
			// Self::deposit_event(Event::Committed { hash: commitment_hash, data: bundle });

			// TODO: persist ABI-encoded leaves to off-chain storage
			// see here: https://github.com/JoshOrndorff/recipes/blob/master/text/off-chain-workers/indexing.md#writing-to-off-chain-storage-from-on-chain-context

			T::WeightInfo::on_initialize(message_count as u32, average_payload_size)
		}

		// TODO: add another RPC method to construct leaf proofs

		fn make_commitment_hash(bundle: &MessageBundleOf<T>) -> H256 {
			let messages: Vec<Token> = bundle
				.messages
				.iter()
				.map(|message| {
					let message = message;
					Token::Tuple(vec![
						Token::Uint(message.id.into()),
						Token::Address(message.target),
						Token::Bytes(message.payload.to_vec()),
					])
				})
				.collect();
			let commitment = ethabi::encode(&vec![Token::Tuple(vec![
				Token::Uint(bundle.source_channel_id.into()),
				Token::Uint(bundle.nonce.into()),
				Token::Array(messages),
			])]);
			<T as Config>::Hashing::hash(&commitment)
		}

		fn average_payload_size(
			messages: &[EnqueuedMessage<T::AccountId, T::MaxMessagePayloadSize>],
		) -> u32 {
			let sum: usize = messages.iter().fold(0, |acc, x| acc + (*x).message.payload.len());
			// We overestimate message payload size rather than underestimate.
			// So add 1 here to account for integer division truncation.
			(sum / messages.len()).saturating_add(1) as u32
		}
	}
}
