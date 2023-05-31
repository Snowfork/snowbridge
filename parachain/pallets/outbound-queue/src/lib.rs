#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use codec::{Decode, Encode, MaxEncodedLen};
use ethabi::{self, Token};
use frame_support::{
	ensure,
	storage::StorageStreamIter,
	traits::{EnqueueMessage, Get, ProcessMessage, ProcessMessageError},
	weights::Weight,
	CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use scale_info::TypeInfo;
use snowbridge_core::ParaId;
use sp_core::{RuntimeDebug, H256};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;
use xcm::v3::XcmHash;

use snowbridge_core::{OutboundQueue as OutboundQueueTrait, SubmitError};
use snowbridge_outbound_queue_merkle_proof::merkle_root;

pub use weights::WeightInfo;

/// Aggregate message origin for the `MessageQueue` pallet.
#[derive(Encode, Decode, Clone, MaxEncodedLen, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AggregateMessageOrigin {
	#[codec(index = 0)]
	Parachain(ParaId),
}

// Message which is awaiting processing
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
pub struct EnqueuedMessage {
	/// XCM Hash
	pub xcm_hash: XcmHash,
	/// ID of source parachain
	pub origin: ParaId,
	/// Handler to dispatch the message to
	pub handler: u16,
	/// Payload for target application.
	pub payload: Vec<u8>,
}

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
pub struct Message {
	/// ID of source parachain
	origin: ParaId,
	/// Unique nonce to prevent replaying messages
	#[codec(compact)]
	nonce: u64,
	/// Handler to dispatch the message to
	handler: u16,
	/// Payload for target application.
	payload: Vec<u8>,
}

impl Into<Token> for Message {
	fn into(self) -> Token {
		Token::Tuple(vec![
			Token::Uint(u32::from(self.origin).into()),
			Token::Uint(self.nonce.into()),
			Token::Uint(self.handler.into()),
			Token::Bytes(self.payload.to_vec()),
		])
	}
}

/// The maximal length of a UMP message.
pub type MaxEnqueuedMessageSizeOf<T> =
	<<T as Config>::MessageQueue as EnqueueMessage<AggregateMessageOrigin>>::MaxMessageLen;

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

		type MessageQueue: EnqueueMessage<AggregateMessageOrigin>;

		/// Max bytes in a message payload
		#[pallet::constant]
		type MaxMessagePayloadSize: Get<u32>;

		/// Max number of messages processed per block
		#[pallet::constant]
		type MaxMessagesPerBlock: Get<u32>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MessageQueued {
			/// ID of the XCM message
			xcm_hash: XcmHash,
		},
		MessageAccepted {
			/// ID of the XCM message
			xcm_hash: XcmHash,
			/// The nonce assigned to this message
			nonce: u64,
		},
		MessagesCommitted {
			/// Merkle root of the committed messages
			root: H256,
			/// number of committed messages
			count: u64,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message is too large
		MessageTooLarge,
	}

	/// Messages to be committed every block. This storage value is killed at the start of every
	/// block, so should never go into block PoV.
	///
	/// Is never read in the runtime, only by offchain code.
	///
	/// Inspired by the `frame_system::Pallet::Events` storage value
	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Messages<T: Config> = StorageValue<_, Vec<Box<Message>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type MessageLeaves<T: Config> = StorageValue<_, Vec<H256>, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, ParaId, u64, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Remove storage from previous block
			Messages::<T>::kill();
			MessageLeaves::<T>::kill();
			return T::WeightInfo::on_finalize()
		}

		fn on_finalize(_: T::BlockNumber) {
			Self::commit_messages();
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn do_process_message(mut message: &[u8]) -> Result<bool, ProcessMessageError> {
			let enqueued_message: EnqueuedMessage =
				EnqueuedMessage::decode(&mut message).map_err(|_| ProcessMessageError::Corrupt)?;

			let next_nonce = Nonce::<T>::get(enqueued_message.origin).saturating_add(1);

			let message: Message = Message {
				origin: enqueued_message.origin,
				nonce: next_nonce,
				handler: enqueued_message.handler,
				payload: enqueued_message.payload,
			};

			let message_abi_encoded = ethabi::encode(&vec![message.clone().into()]);
			let message_abi_encoded_hash = <T as Config>::Hashing::hash(&message_abi_encoded);

			Messages::<T>::append(Box::new(message));
			MessageLeaves::<T>::append(message_abi_encoded_hash);
			Nonce::<T>::set(enqueued_message.origin, next_nonce);

			Self::deposit_event(Event::MessageAccepted {
				xcm_hash: enqueued_message.xcm_hash,
				nonce: next_nonce,
			});

			Ok(true)
		}

		pub(crate) fn commit_messages() {
			let messages_count = MessageLeaves::<T>::decode_len().unwrap_or_default() as u64;
			if messages_count == 0 {
				return
			}

			// Create merkle root of messages
			let messages_root =
				merkle_root::<<T as Config>::Hashing, _>(MessageLeaves::<T>::stream_iter());

			// Insert merkle root into the block header
			<frame_system::Pallet<T>>::deposit_log(sp_runtime::DigestItem::Other(
				messages_root.to_fixed_bytes().into(),
			));

			Self::deposit_event(Event::MessagesCommitted {
				root: messages_root,
				count: messages_count,
			});
		}

		fn prepare_enqueued_message(
			xcm_hash: XcmHash,
			origin: ParaId,
			handler: u16,
			payload: &[u8],
		) -> Result<BoundedVec<u8, MaxEnqueuedMessageSizeOf<T>>, SubmitError> {
			// The inner payload should not be too large
			ensure!(
				payload.len() < T::MaxMessagePayloadSize::get() as usize,
				SubmitError::MessageTooLarge
			);
			let message: EnqueuedMessage = EnqueuedMessage {
				xcm_hash,
				origin: origin.clone(),
				handler,
				payload: payload.into(),
			};
			// The whole message should not be too large
			let message = message.encode().try_into().map_err(|_| SubmitError::MessageTooLarge)?;
			Ok(message)
		}
	}

	impl<T: Config> OutboundQueueTrait for Pallet<T> {
		/// Ensure that the message isn't too large
		fn validate(
			xcm_hash: XcmHash,
			origin: ParaId,
			handler: u16,
			payload: &[u8],
		) -> Result<(), SubmitError> {
			Self::prepare_enqueued_message(xcm_hash, origin, handler, payload).map(|_| ())
		}

		/// Submit message on the outbound channel
		fn submit(
			xcm_hash: XcmHash,
			origin: ParaId,
			handler: u16,
			payload: &[u8],
		) -> Result<(), SubmitError> {
			let message = Self::prepare_enqueued_message(xcm_hash, origin, handler, payload)?;
			T::MessageQueue::enqueue_message(
				message.as_bounded_slice(),
				AggregateMessageOrigin::Parachain(origin),
			);
			Self::deposit_event(Event::MessageQueued { xcm_hash });
			Ok(())
		}
	}

	impl<T: Config> ProcessMessage for Pallet<T> {
		type Origin = AggregateMessageOrigin;
		fn process_message(
			message: &[u8],
			_: Self::Origin,
			meter: &mut frame_support::weights::WeightMeter,
		) -> Result<bool, ProcessMessageError> {
			ensure!(
				MessageLeaves::<T>::decode_len().unwrap_or(0) <
					T::MaxMessagesPerBlock::get() as usize,
				ProcessMessageError::Yield
			);

			let weight = T::WeightInfo::do_process_message();
			if !meter.check_accrue(weight) {
				return Err(ProcessMessageError::Overweight(weight))
			}

			Self::do_process_message(message)
		}
	}
}
