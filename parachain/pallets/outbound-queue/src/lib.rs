// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
#![cfg_attr(not(feature = "std"), no_std)]

pub mod api;
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

use snowbridge_core::{
	ContractId, OutboundMessage, OutboundQueue as OutboundQueueTrait, SubmitError,
};
use snowbridge_outbound_queue_merkle_tree::merkle_root;

pub use snowbridge_outbound_queue_merkle_tree::MerkleProof;
pub use weights::WeightInfo;

/// Aggregate message origin for the `MessageQueue` pallet.
#[derive(Encode, Decode, Clone, MaxEncodedLen, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AggregateMessageOrigin {
	#[codec(index = 0)]
	Parachain(ParaId),
}

/// Message which is awaiting processing in the MessageQueue pallet
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct EnqueuedMessage {
	/// Message ID (usually hash of message)
	pub id: H256,
	/// ID of source parachain
	pub origin: ParaId,
	/// The receiving gateway contract
	pub command: H256,
	/// Payload for target application.
	pub params: Vec<u8>,
}

/// Message which has been assigned a nonce and will be committed at the end of a block
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
pub struct Message {
	/// ID of source parachain
	origin: ParaId,
	/// Unique nonce to prevent replaying messages
	nonce: u64,
	/// Command to execute in the Gateway contract
	command: H256,
	/// Payload for target application.
	params: Vec<u8>,
}

/// Convert message into an ABI-encoded form for delivery to the InboundQueue contract on Ethereum
impl Into<Token> for Message {
	fn into(self) -> Token {
		Token::Tuple(vec![
			Token::Uint(u32::from(self.origin).into()),
			Token::Uint(self.nonce.into()),
			Token::FixedBytes(self.command.to_fixed_bytes().into()),
			Token::Bytes(self.params.to_vec()),
		])
	}
}

impl From<u32> for AggregateMessageOrigin {
	fn from(value: u32) -> Self {
		AggregateMessageOrigin::Parachain(value.into())
	}
}

/// The maximal length of an enqueued message, as determined by the MessageQueue pallet
pub type MaxEnqueuedMessageSizeOf<T> =
	<<T as Config>::MessageQueue as EnqueueMessage<AggregateMessageOrigin>>::MaxMessageLen;

pub use pallet::*;

pub const LOG_TARGET: &str = "snowbridge-outbound-queue";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use bp_runtime::{BasicOperatingMode, OwnedBridgeModule};

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
		/// Message has been queued and will be processed in the future
		MessageQueued {
			/// ID of the message. Usually the XCM message hash.
			id: H256,
		},
		/// Message will be committed at the end of current block. From now on, to track the
		/// progress the message, use the `nonce` of `id`.
		MessageAccepted {
			/// ID of the message
			id: H256,
			/// The nonce assigned to this message
			nonce: u64,
		},
		/// Some messages have been committed
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

	/// Messages to be committed in the current block. This storage value is killed in
	/// `on_initialize`, so should never go into block PoV.
	///
	/// Is never read in the runtime, only by offchain code.
	///
	/// Inspired by the `frame_system::Pallet::Events` storage value
	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Messages<T: Config> = StorageValue<_, Vec<Message>, ValueQuery>;

	/// Hashes of the ABI-encoded messages in the [`Messages`] storage value. Used to generate a
	/// merkle root during `on_finalize`. This storage value is killed in
	/// `on_initialize`, so should never go into block PoV.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn message_leaves)]
	pub(super) type MessageLeaves<T: Config> = StorageValue<_, Vec<H256>, ValueQuery>;

	/// The current nonce for each message origin
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, ParaId, u64, ValueQuery>;

	/// Optional pallet owner.
	/// Pallet owner has a right to halt all pallet operations and then resume them. If it is
	/// `None`, then there are no direct ways to halt/resume pallet operations, but other
	/// runtime methods may still be used to do that (i.e. democracy::referendum to update halt
	/// flag directly or call the `halt_operations`).
	#[pallet::storage]
	pub type PalletOwner<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	/// The current operating mode of the pallet.
	/// Depending on the mode either all, or no transactions will be allowed.
	#[pallet::storage]
	pub type PalletOperatingMode<T: Config> = StorageValue<_, BasicOperatingMode, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Remove storage from previous block
			Messages::<T>::kill();
			MessageLeaves::<T>::kill();
			// Reserve some weight for the `on_finalize` handler
			return T::WeightInfo::on_finalize()
		}

		fn on_finalize(_: T::BlockNumber) {
			Self::commit_messages();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Change `PalletOwner`.
		/// May only be called either by root, or by `PalletOwner`.
		#[pallet::call_index(0)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_owner(origin: OriginFor<T>, new_owner: Option<T::AccountId>) -> DispatchResult {
			<Self as OwnedBridgeModule<_>>::set_owner(origin, new_owner)
		}

		/// Halt or resume all pallet operations.
		/// May only be called either by root, or by `PalletOwner`.
		#[pallet::call_index(1)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_operating_mode(
			origin: OriginFor<T>,
			operating_mode: BasicOperatingMode,
		) -> DispatchResult {
			<Self as OwnedBridgeModule<_>>::set_operating_mode(origin, operating_mode)
		}
	}

	impl<T: Config> OwnedBridgeModule<T> for Pallet<T> {
		const LOG_TARGET: &'static str = LOG_TARGET;
		type OwnerStorage = PalletOwner<T>;
		type OperatingMode = BasicOperatingMode;
		type OperatingModeStorage = PalletOperatingMode<T>;
	}

	impl<T: Config> Pallet<T> {
		/// Generate a messages commitment and insert it into the header digest
		pub(crate) fn commit_messages() {
			let count = MessageLeaves::<T>::decode_len().unwrap_or_default() as u64;
			if count == 0 {
				return
			}

			// Create merkle root of messages
			let root = merkle_root::<<T as Config>::Hashing, _>(MessageLeaves::<T>::stream_iter());

			// Insert merkle root into the header digest
			<frame_system::Pallet<T>>::deposit_log(sp_runtime::DigestItem::Other(
				root.to_fixed_bytes().into(),
			));

			Self::deposit_event(Event::MessagesCommitted { root, count });
		}

		/// Process a message delivered by the MessageQueue pallet
		pub(crate) fn do_process_message(mut message: &[u8]) -> Result<bool, ProcessMessageError> {
			let enqueued_message: EnqueuedMessage =
				EnqueuedMessage::decode(&mut message).map_err(|_| ProcessMessageError::Corrupt)?;

			let next_nonce = Nonce::<T>::get(enqueued_message.origin).saturating_add(1);

			let message: Message = Message {
				origin: enqueued_message.origin,
				nonce: next_nonce,
				command: enqueued_message.command,
				params: enqueued_message.params,
			};

			let message_abi_encoded = ethabi::encode(&vec![message.clone().into()]);
			let message_abi_encoded_hash = <T as Config>::Hashing::hash(&message_abi_encoded);

			Messages::<T>::append(Box::new(message));
			MessageLeaves::<T>::append(message_abi_encoded_hash);
			Nonce::<T>::set(enqueued_message.origin, next_nonce);

			Self::deposit_event(Event::MessageAccepted {
				id: enqueued_message.id,
				nonce: next_nonce,
			});

			Ok(true)
		}
	}

	/// A message which can be accepted by the [`OutboundQueue`]
	#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound)]
	pub struct OutboundQueueTicket<MaxMessageSize: Get<u32>> {
		id: H256,
		origin: ParaId,
		message: BoundedVec<u8, MaxMessageSize>,
	}

	impl<T: Config> OutboundQueueTrait for Pallet<T> {
		type Ticket = OutboundQueueTicket<MaxEnqueuedMessageSizeOf<T>>;

		fn validate(message: &OutboundMessage) -> Result<Self::Ticket, SubmitError> {
			// The inner payload should not be too large
			ensure!(
				message.params.len() < T::MaxMessagePayloadSize::get() as usize,
				SubmitError::MessageTooLarge
			);
			let message: EnqueuedMessage = EnqueuedMessage {
				id: message.id,
				origin: message.origin,
				command: message.command,
				params: message.params.clone().into(),
			};
			// The whole message should not be too large
			let encoded = message.encode().try_into().map_err(|_| SubmitError::MessageTooLarge)?;

			let ticket =
				OutboundQueueTicket { id: message.id, origin: message.origin, message: encoded };
			Ok(ticket)
		}

		fn submit(ticket: Self::Ticket) -> Result<(), SubmitError> {
			Self::ensure_not_halted().map_err(|_| SubmitError::BridgeHalted)?;
			T::MessageQueue::enqueue_message(
				ticket.message.as_bounded_slice(),
				AggregateMessageOrigin::Parachain(ticket.origin),
			);
			Self::deposit_event(Event::MessageQueued { id: ticket.id });
			Ok(())
		}
	}

	impl<T: Config> ProcessMessage for Pallet<T> {
		type Origin = AggregateMessageOrigin;
		fn process_message(
			message: &[u8],
			_: Self::Origin,
			meter: &mut frame_support::weights::WeightMeter,
			_: &mut [u8; 32],
		) -> Result<bool, ProcessMessageError> {
			// Yield if we don't want to accept any more messages in the current block.
			// There is hard limit to ensure the weight of `on_finalize` is bounded.
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
