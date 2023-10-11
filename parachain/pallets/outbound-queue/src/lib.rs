// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

//! Pallet for committing outbound messages for delivery to Ethereum
//!
//! The message submission pipeline works like this:
//! 1. The message is first validated via [`OutboundQueue::validate`]
//! 2. The message is then enqueued for processing via [`OutboundQueue::submit`]
//! 3. The message queue is maintained by the external [`MessageQueue`] pallet
//! 4. [`MessageQueue`] delivers messages back to this pallet via `ProcessMessage::process_message`
//! 5. The message is processed in `do_process_message` a. Assigned a nonce b. ABI-encoded, hashed,
//!    and stored in the `Leaves` vector
//! 6. At the end of the block, a merkle root is constructed from all the leaves in `Leaves`.
//! 7. This merkle root is inserted into the parachain header as a digest item
//!
//! On the Ethereum side, the message root is ultimately the thing being
//! by the Polkadot light client.
#![cfg_attr(not(feature = "std"), no_std)]
pub mod api;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use codec::{Decode, Encode};
use ethabi::{self};
use frame_support::{
	ensure,
	storage::StorageStreamIter,
	traits::{tokens::Balance, EnqueueMessage, Get, ProcessMessage, ProcessMessageError},
	weights::Weight,
};
use frame_system::EnsureRoot;
use snowbridge_core::ParaId;
use sp_core::H256;
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use snowbridge_core::{
	outbound::{
		AggregateMessageOrigin, EnqueuedMessage, GasMeter, Message, MessageHash,
		OutboundQueue as OutboundQueueTrait, OutboundQueueTicket, PreparedMessage, SubmitError,
	},
	BasicOperatingMode, BridgeModule,
};
use snowbridge_outbound_queue_merkle_tree::merkle_root;

pub use snowbridge_outbound_queue_merkle_tree::MerkleProof;
pub use weights::WeightInfo;

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

		type GasMeter: GasMeter;

		type Balance: Balance;

		/// The fee charged locally for accepting a message.
		type Fee: Get<Self::Balance>;

		/// The reward in ether paid to relayers for sending a message to Ethereum
		type Reward: Get<u128>;

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
		/// Set OperatingMode
		OperatingModeSet { operating_mode: BasicOperatingMode },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message is too large
		MessageTooLarge,
	}

	/// Messages to be committed in the current block. This storage value is killed in
	/// `on_initialize`, so should never go into block PoV.
	///
	/// Is never read in the runtime, only by offchain message relayers.
	///
	/// Inspired by the `frame_system::Pallet::Events` storage value
	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Messages<T: Config> = StorageValue<_, Vec<PreparedMessage>, ValueQuery>;

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

	/// The current operating mode of the pallet.
	/// Depending on the mode either all, or no transactions will be allowed.
	#[pallet::storage]
	pub type PalletOperatingMode<T: Config> = StorageValue<_, BasicOperatingMode, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		fn on_initialize(_: BlockNumberFor<T>) -> Weight {
			// Remove storage from previous block
			Messages::<T>::kill();
			MessageLeaves::<T>::kill();
			// Reserve some weight for the `on_finalize` handler
			T::WeightInfo::on_finalize()
		}

		fn on_finalize(_: BlockNumberFor<T>) {
			Self::commit_messages();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Halt or resume all pallet operations.
		/// May only be called either by root, or by `PalletOwner`.
		#[pallet::call_index(0)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_operating_mode(
			origin: OriginFor<T>,
			operating_mode: BasicOperatingMode,
		) -> DispatchResult {
			<Self as BridgeModule<_>>::set_operating_mode(origin, operating_mode)?;
			Self::deposit_event(Event::OperatingModeSet { operating_mode });
			Ok(())
		}
	}

	impl<T: Config> BridgeModule<T> for Pallet<T> {
		type OperatingMode = BasicOperatingMode;
		type OperatingModeStorage = PalletOperatingMode<T>;
		type AllowedHaltOrigin = EnsureRoot<T::AccountId>;
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

			let command = enqueued_message.command.index();
			let params = enqueued_message.command.abi_encode();
			let max_dispatch_gas =
				T::GasMeter::measure_maximum_required_gas(&enqueued_message.command) as u128;
			let reward = T::Reward::get();

			// Construct a prepared message, which when ABI-encoded is what the
			// other side of the bridge will verify.
			let message: PreparedMessage = PreparedMessage {
				origin: enqueued_message.origin,
				nonce: next_nonce,
				command,
				params,
				max_dispatch_gas,
				reward,
			};

			// ABI-encode and hash the prepared message
			let message_abi_encoded = ethabi::encode(&[message.clone().into()]);
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

	impl<T: Config> OutboundQueueTrait for Pallet<T> {
		type Ticket = OutboundQueueTicket<MaxEnqueuedMessageSizeOf<T>>;
		type Balance = T::Balance;

		fn validate(message: &Message) -> Result<(Self::Ticket, Self::Balance), SubmitError> {
			// The inner payload should not be too large
			let payload = message.command.abi_encode();

			// Create a message id for tracking progress in submission pipeline
			let message_id: MessageHash = sp_io::hashing::blake2_256(&(message.encode())).into();

			ensure!(
				payload.len() < T::MaxMessagePayloadSize::get() as usize,
				SubmitError::MessageTooLarge
			);
			let command = message.command.clone();
			let enqueued_message: EnqueuedMessage =
				EnqueuedMessage { id: message_id, origin: message.origin, command };
			// The whole message should not be too large
			let encoded =
				enqueued_message.encode().try_into().map_err(|_| SubmitError::MessageTooLarge)?;

			let ticket =
				OutboundQueueTicket { id: message_id, origin: message.origin, message: encoded };
			Ok((ticket, T::Fee::get()))
		}

		fn submit(ticket: Self::Ticket) -> Result<MessageHash, SubmitError> {
			// Make sure the bridge not halted
			Self::ensure_not_halted().map_err(|_| SubmitError::BridgeHalted)?;
			T::MessageQueue::enqueue_message(
				ticket.message.as_bounded_slice(),
				AggregateMessageOrigin::Parachain(ticket.origin),
			);
			Self::deposit_event(Event::MessageQueued { id: ticket.id });
			Ok(ticket.id)
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
			// Make sure the bridge not halted
			Self::ensure_not_halted().map_err(|_| ProcessMessageError::Yield)?;
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
