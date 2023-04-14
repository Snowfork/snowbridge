#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use codec::{Decode, Encode, MaxEncodedLen};
use ethabi::{self, Token};
use frame_support::{
	dispatch::DispatchResult, ensure, traits::Get, weights::Weight, BoundedVec, CloneNoBound,
	PartialEqNoBound, RuntimeDebugNoBound,
};
use polkadot_parachain::primitives::Id as ParaId;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_io::offchain_index::set;
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use snowbridge_core::types::AuxiliaryDigestItem;
use snowbridge_outbound_queue_merkle_proof::merkle_root;

pub use weights::WeightInfo;

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, MaxEncodedLen, TypeInfo,
)]
#[scale_info(skip_type_params(M))]
pub struct Message<M: Get<u32>> {
	/// ID of source parachain
	origin: ParaId,
	/// Unique nonce to prevent replaying messages
	#[codec(compact)]
	nonce: u64,
	/// Handler to dispatch the message to
	handler: u16,
	/// Payload for target application.
	payload: BoundedVec<u8, M>,
}

impl<M: Get<u32>> Into<Token> for Message<M> {
	fn into(self) -> Token {
		Token::Tuple(vec![
			Token::Uint(u32::from(self.origin).into()),
			Token::Uint(self.nonce.into()),
			Token::Uint(self.handler.into()),
			Token::Bytes(self.payload.to_vec()),
		])
	}
}

// base_weight=(0.75*0.5)*(10**12)=375_000_000_000
// we leave the extra 10_000_000_000/375_000_000_000=2.66% as margin
// so we can use at most 365000000000 for the commit call
// need to rerun benchmarks later to get weight based on the worst case:
// MaxMessagesPerCommit=20 and MaxMessagePayloadSize=256
pub const MINIMUM_WEIGHT_REMAIN_IN_BLOCK: Weight = Weight::from_parts(10_000_000_000, 0);

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
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
		Committed { hash: H256, data: Vec<Message<T::MaxMessagePayloadSize>> },
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
	pub(super) type MessageQueue<T: Config> = StorageValue<
		_,
		BoundedVec<Message<T::MaxMessagePayloadSize>, T::MaxMessagesPerCommit>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, ParaId, u64, ValueQuery>;

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
		// Generate a message commitment when the chain is idle with enough remaining weight
		// The commitment hash is included in an [`AuxiliaryDigestItem`] in the block header,
		// with the corresponding commitment is persisted offchain.
		fn on_idle(_n: T::BlockNumber, total_weight: Weight) -> Weight {
			let weight_remaining = total_weight.saturating_sub(T::WeightInfo::on_commit(
				T::MaxMessagesPerCommit::get(),
				T::MaxMessagePayloadSize::get(),
			));
			if weight_remaining.ref_time() <= MINIMUM_WEIGHT_REMAIN_IN_BLOCK.ref_time() {
				return total_weight
			}
			Self::commit(total_weight)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Submit message on the outbound channel
		pub fn submit(origin: &ParaId, handler: u16, payload: &[u8]) -> DispatchResult {
			ensure!(
				<MessageQueue<T>>::decode_len().unwrap_or(0) <
					T::MaxMessagesPerCommit::get() as usize,
				Error::<T>::QueueSizeLimitReached,
			);

			let message_payload =
				payload.to_vec().try_into().map_err(|_| Error::<T>::PayloadTooLarge)?;
			let nonce = <Nonce<T>>::get(origin);
			let next_nonce = nonce.checked_add(1).ok_or(Error::<T>::Overflow)?;

			<MessageQueue<T>>::try_append(Message {
				origin: origin.clone(),
				nonce,
				handler,
				payload: message_payload,
			})
			.map_err(|_| Error::<T>::QueueSizeLimitReached)?;
			Self::deposit_event(Event::MessageAccepted(nonce));

			<Nonce<T>>::set(origin, next_nonce);

			Ok(())
		}

		/// Commit messages enqueued on the outbound channel.
		/// Find the Merkle root of all of the messages in the queue. (TODO: take a sublist, later
		/// use weights to determine the size of the sublist). Use ethabi-encoded messages as the
		/// leaves of the Merkle tree. Then:
		/// - Store the commitment hash on the parachain for the Ethereum light client to query.
		/// - Emit an event with the commitment hash and SCALE-encoded message bundles for a
		/// relayer to read.
		/// - Persist the ethabi-encoded message bundles to off-chain storage.
		pub fn commit(_total_weight: Weight) -> Weight {
			// TODO: SNO-310 consider using mutate here. If some part of emitting message bundles
			// fails, we don't want the MessageQueue to be empty.
			let message_queue = <MessageQueue<T>>::take();
			if message_queue.is_empty() {
				return T::WeightInfo::on_commit_no_messages()
			}

			// Store these to return the on_commit weight
			let message_count = message_queue.len() as u32;
			let average_payload_size = Self::average_payload_size(&message_queue);

			let eth_messages: Vec<Vec<u8>> = message_queue
				.clone()
				.into_iter()
				.map(|msg| ethabi::encode(&vec![msg.into()]))
				.collect();

			let commitment_hash =
				merkle_root::<<T as Config>::Hashing, Vec<Vec<u8>>, Vec<u8>>(eth_messages.clone());

			let digest_item = AuxiliaryDigestItem::Commitment(commitment_hash.clone()).into();
			<frame_system::Pallet<T>>::deposit_log(digest_item);

			Self::deposit_event(Event::Committed {
				hash: commitment_hash,
				data: message_queue.to_vec(),
			});

			set(commitment_hash.as_bytes(), &eth_messages.encode());

			return T::WeightInfo::on_commit(message_count, average_payload_size)
		}

		fn average_payload_size(messages: &[Message<T::MaxMessagePayloadSize>]) -> u32 {
			let sum: usize = messages.iter().fold(0, |acc, x| acc + (*x).payload.len());
			// We overestimate message payload size rather than underestimate.
			// So add 1 here to account for integer division truncation.
			(sum / messages.len()).saturating_add(1) as u32
		}
	}
}
