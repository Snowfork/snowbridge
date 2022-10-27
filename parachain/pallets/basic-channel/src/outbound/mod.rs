pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use codec::{Decode, Encode, MaxEncodedLen};
use ethabi::{self, Token};
use frame_support::{
	dispatch::DispatchResult, ensure, traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound,
	RuntimeDebugNoBound,
};
use scale_info::TypeInfo;
use sp_core::{H160, H256};
use sp_runtime::traits::{Hash, Zero};

use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, prelude::*};

use sp_io::offchain_index::set;

use snowbridge_core::{types::AuxiliaryDigestItem, ChannelId};

use snowbridge_basic_channel_merkle_proof::merkle_root;

pub use weights::WeightInfo;

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

impl<M: Get<u32>> Into<Token> for Message<M> {
	fn into(self) -> Token {
		Token::Tuple(vec![
			Token::Uint(self.id.into()),
			Token::Address(self.target),
			Token::Bytes(self.payload.to_vec()),
		])
	}
}

pub type EnqueuedMessageOf<T> =
	EnqueuedMessage<<T as frame_system::Config>::AccountId, <T as Config>::MaxMessagePayloadSize>;

/// Wire-format for committed messages
#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, MaxEncodedLen, TypeInfo,
)]
#[scale_info(skip_type_params(M, N))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct MessageBundle<AccountId, M: Get<u32>, N: Get<u32>>
where
	AccountId: Encode + Decode + Clone + PartialEq + Debug + MaxEncodedLen + TypeInfo,
{
	source_channel_id: u8,
	account: AccountId,
	/// Unique nonce to prevent replaying bundles
	#[codec(compact)]
	nonce: u64,
	messages: BoundedVec<Message<M>, N>,
}

impl<AccountId, M: Get<u32>, N: Get<u32>> Into<Token> for MessageBundle<AccountId, M, N>
where
	AccountId: AsRef<[u8]> + Encode + Decode + Clone + PartialEq + Debug + MaxEncodedLen + TypeInfo,
{
	fn into(self) -> Token {
		Token::Tuple(vec![
			Token::Uint(self.source_channel_id.into()),
			Token::FixedBytes(self.account.as_ref().into()),
			Token::Uint(self.nonce.into()),
			Token::Array(self.messages.into_iter().map(|message| message.into()).collect()),
		])
	}
}

pub type MessageBundleOf<T> = MessageBundle<
	<T as frame_system::Config>::AccountId,
	<T as Config>::MaxMessagePayloadSize,
	<T as Config>::MaxMessagesPerCommit,
>;

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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Hashing: Hash<Output = H256>;

		/// Max bytes in a message payload
		#[pallet::constant]
		type MaxMessagePayloadSize: Get<u32>;

		/// Max number of messages per commitment
		#[pallet::constant]
		type MaxMessagesPerCommit: Get<u32>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MessageAccepted(u64),
		Committed { hash: H256, data: Vec<MessageBundleOf<T>> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message payload exceeds byte limit.
		PayloadTooLarge,
		/// No more messages can be queued for the channel during this commit cycle.
		QueueSizeLimitReached,
		/// Cannot increment nonce
		Overflow,
	}

	/// Interval between commitments
	#[pallet::storage]
	#[pallet::getter(fn interval)]
	pub(super) type Interval<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Messages waiting to be committed.
	#[pallet::storage]
	pub(super) type MessageQueue<T: Config> =
		StorageValue<_, BoundedVec<EnqueuedMessageOf<T>, T::MaxMessagesPerCommit>, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u64, ValueQuery>;

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
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
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

	impl<T: Config> Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
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

		/// Commit messages enqueued on the outbound channel.
		/// For every account id in the enqueued messages, create a message bundle containing the
		/// messages for that account. Hash the ethabi encoding of these message bundles in a
		/// Merkle tree to produce a commitment hash. Then:
		/// - Store the commitment hash on the parachain for the Ethereum light client to query.
		/// - Emit an event with the commitment hash and SCALE-encoded message bundles for a
		/// relayer to read.
		/// - Persist the ethabi-encoded message bundles to off-chain storage.
		fn commit() -> Weight {
			// TODO: SNO-310 consider using mutate here. If some part of emitting message bundles
			// fails, we don't want the MessageQueue to be empty.
			let message_queue = <MessageQueue<T>>::take();
			if message_queue.is_empty() {
				return T::WeightInfo::on_initialize_no_messages();
			}

			// Store these for the on_initialize call at the end
			let message_count = message_queue.len() as u32;
			let average_payload_size = Self::average_payload_size(&message_queue);

			let message_bundles = Self::make_message_bundles(message_queue);
			let eth_message_bundles: Vec<Vec<u8>> = message_bundles
				.clone()
				.into_iter()
				.map(|bundle| ethabi::encode(&vec![bundle.into()]))
				.collect();

			let commitment_hash = merkle_root::<<T as Config>::Hashing, Vec<Vec<u8>>, Vec<u8>>(
				eth_message_bundles.clone(),
			);

			let digest_item =
				AuxiliaryDigestItem::Commitment(ChannelId::Basic, commitment_hash.clone()).into();
			<frame_system::Pallet<T>>::deposit_log(digest_item);

			Self::deposit_event(Event::Committed {
				hash: commitment_hash,
				data: message_bundles.clone(),
			});

			set(commitment_hash.as_bytes(), &eth_message_bundles.encode());

			T::WeightInfo::on_initialize(message_count, average_payload_size)
		}

		fn make_message_bundles(
			message_queue: BoundedVec<EnqueuedMessageOf<T>, <T as Config>::MaxMessagesPerCommit>,
		) -> Vec<MessageBundleOf<T>> {
			let account_message_map: BTreeMap<
				T::AccountId,
				BoundedVec<Message<T::MaxMessagePayloadSize>, T::MaxMessagesPerCommit>,
			> = message_queue.into_iter().fold(
				BTreeMap::new(),
				|mut messages_for_accounts, enqueued_message| {
					let (account, message) = (enqueued_message.account, enqueued_message.message);

					if let Some(messages) = messages_for_accounts.get_mut(&account) {
						// We should be able to safely ignore the result here, since we can't have
						// more messages for a single account than we have in the message queue
						messages.try_push(message).unwrap();
					} else {
						let mut messages = BoundedVec::default();
						// Safe to ignore the result because we just created the empty BoundedVec
						messages.try_push(message).unwrap();
						messages_for_accounts.insert(account, messages);
					}

					messages_for_accounts
				},
			);

			let mut message_bundles: Vec<MessageBundleOf<T>> = Vec::new();
			for (account, messages) in account_message_map {
				let next_nonce = <Nonce<T>>::mutate(&account, |nonce| {
					*nonce = nonce.saturating_add(1);
					*nonce
				});
				let bundle: MessageBundleOf<T> = MessageBundle {
					source_channel_id: ChannelId::Basic as u8,
					account,
					nonce: next_nonce,
					messages,
				};
				message_bundles.push(bundle);
			}

			message_bundles
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
