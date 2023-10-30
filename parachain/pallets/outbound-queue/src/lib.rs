// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Pallet for committing outbound messages for delivery to Ethereum
//!
//! # Overview
//!
//! The message submission pipeline works like this:
//! 1. The message is first validated via the implementation for
//!    [`snowbridge_core::outbound::SendMessage::validate`]
//! 2. The message is then enqueued for later processing via the implementation
//!    for [`snowbridge_core::outbound::SendMessage::deliver`]
//! 3. The underlying message queue is implemented by [`Config::MessageQueue`]
//! 4. The message queue delivers messages back to this pallet via
//!    the implementation for [`frame_support::traits::ProcessMessage::process_message`]
//! 5. The message is processed in `Pallet::do_process_message`:
//!    a. Assigned a nonce
//!    b. ABI-encoded, hashed, and stored in the `MessageLeaves` vector
//! 6. At the end of the block, a merkle root is constructed from all the
//!    leaves in `MessageLeaves`.
//! 7. This merkle root is inserted into the parachain header as a digest item
//! 8. Offchain relayers can read the committed message from the `Messages`
//!    storage item.
//!
//! On the Ethereum side, the message root is ultimately the thing being
//! by the Polkadot light client.
//!
//! # Message Priorities
//!
//! Within the message submission pipeline, messages have different priorities,
//! which results in differing processing behavior.
//!
//! All outgoing messages are buffered in the `MessageQueue` pallet, however
//! Governance commands are always processed before lower priority commands
//!
//! The processing of governance commands can never be halted. This effectively
//! allows us to pause processing of normal user messages while still allowing
//! governance commands to be sent to Ethereum.
//!
//! # Fees
//!
//! An upfront fee must be paid for delivering a message. This fee covers several
//! components:
//! 1. The weight of processing the message locally
//! 2. The gas refund paid out to relayers for message submission
//! 3. An additional reward paid out to relayers for message submission
//!
//! Messages are weighed to determine the maximum amount of gas they could
//! consume on Ethereum. Using this upper bound, a final fee can be calculated.
//!
//! The fee calculation also requires the following parameters:
//! * ETH/DOT exchange rate
//! * Ether fee per unit of gas
//!
//! By design, it is expected that governance should manually update these
//! parameters every few weeks using the [`Call::set_fee_config`] extrinsic.
//!
//! ## Fee Computation Function
//!
//! ```text
//! LocalFee(Message) = ProcessingWeight(Message)
//! RemoteFee(Message) = MaxGasRequired(Message) * FeePerGas + Reward
//! Fee(Message) = LocalFee(Message) + (RemoteFee(Message) / Ratio("ETH/DOT"))
//! ```
//!
//! # Message Types
//!
//! The naming of message types indicates their processing state:
//!
//! 1. `Message`: Message initially received
//! 2. `QueuedMessage`: Message queued for eventual commitment
//! 3. `CommittedMessage`: Message has been committed for delivery
//!
#![cfg_attr(not(feature = "std"), no_std)]
pub mod api;
pub mod process_message_impl;
pub mod send_message_impl;
pub mod types;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

use codec::Decode;
use frame_support::{
	ensure,
	storage::StorageStreamIter,
	traits::{tokens::Balance, EnqueueMessage, Get, ProcessMessageError},
	weights::{Weight, WeightToFee},
};
use snowbridge_core::{
	outbound::{
		AggregateMessageOrigin, Command, QueuedMessage, ExportOrigin, Fee, GasMeter,
		VersionedQueuedMessage,
	},
	BasicOperatingMode, ParaId,
};
use snowbridge_outbound_queue_merkle_tree::merkle_root;
pub use snowbridge_outbound_queue_merkle_tree::MerkleProof;
use sp_core::H256;
use sp_runtime::{
	traits::{CheckedDiv, Hash},
	FixedPointNumber,
};
use sp_std::prelude::*;
pub use types::{CommittedMessage, FeeConfigRecord, ProcessMessageOriginOf};
pub use weights::WeightInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::FixedU128;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Hashing: Hash<Output = H256>;

		type MessageQueue: EnqueueMessage<AggregateMessageOrigin>;

		/// Measures the maximum gas used to execute a command on Ethereum
		type GasMeter: GasMeter;

		type Balance: Balance + From<u128>;

		/// Max bytes in a message payload
		#[pallet::constant]
		type MaxMessagePayloadSize: Get<u32>;

		/// Max number of messages processed per block
		#[pallet::constant]
		type MaxMessagesPerBlock: Get<u32>;

		/// The ID of this parachain
		#[pallet::constant]
		type OwnParaId: Get<ParaId>;

		/// Convert a weight value into a deductible fee based.
		type WeightToFee: WeightToFee<Balance = Self::Balance>;

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
		OperatingModeChanged {
			mode: BasicOperatingMode,
		},
		FeeConfigChanged {
			fee_config: FeeConfigRecord,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message is too large
		MessageTooLarge,
		/// The pallet is halted
		Halted,
		// Invalid fee config
		InvalidFeeConfig,
	}

	/// Messages to be committed in the current block. This storage value is killed in
	/// `on_initialize`, so should never go into block PoV.
	///
	/// Is never read in the runtime, only by offchain message relayers.
	///
	/// Inspired by the `frame_system::Pallet::Events` storage value
	#[pallet::storage]
	#[pallet::unbounded]
	pub(super) type Messages<T: Config> = StorageValue<_, Vec<CommittedMessage>, ValueQuery>;

	/// Number of high priority messages that are waiting to be processed.
	/// While this number is greater than zero, processing of lower priority
	/// messages is paused.
	#[pallet::storage]
	pub(super) type PendingHighPriorityMessageCount<T: Config> = StorageValue<_, u32, ValueQuery>;

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
	#[pallet::storage]
	#[pallet::getter(fn operating_mode)]
	pub type OperatingMode<T: Config> = StorageValue<_, BasicOperatingMode, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fee_config)]
	pub type FeeConfig<T: Config> = StorageValue<_, FeeConfigRecord, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		pub operating_mode: BasicOperatingMode,
		pub fee_config: FeeConfigRecord,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			self.fee_config.validate().expect("invalid fee configuration");
			OperatingMode::<T>::put(self.operating_mode);
			FeeConfig::<T>::put(self.fee_config);
		}
	}

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
			T::WeightInfo::commit()
		}

		fn on_finalize(_: BlockNumberFor<T>) {
			Self::commit();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Halt or resume all pallet operations. May only be called by root.
		#[pallet::call_index(0)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_operating_mode(
			origin: OriginFor<T>,
			mode: BasicOperatingMode,
		) -> DispatchResult {
			ensure_root(origin)?;
			OperatingMode::<T>::put(mode);
			Self::deposit_event(Event::OperatingModeChanged { mode });
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_fee_config(origin: OriginFor<T>, fee_config: FeeConfigRecord) -> DispatchResult {
			ensure_root(origin)?;
			fee_config.validate().map_err(|_| Error::<T>::InvalidFeeConfig)?;
			FeeConfig::<T>::put(fee_config);
			Self::deposit_event(Event::FeeConfigChanged { fee_config });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Generate a messages commitment and insert it into the header digest
		pub(crate) fn commit() {
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
		pub(crate) fn do_process_message(
			origin: ProcessMessageOriginOf<T>,
			mut message: &[u8],
		) -> Result<bool, ProcessMessageError> {
			use AggregateMessageOrigin::*;
			use ExportOrigin::*;
			use ProcessMessageError::*;

			// Yield if the maximum number of messages has been processed this block.
			// This ensures that the weight of `on_finalize` has a known maximum bound.
			ensure!(
				MessageLeaves::<T>::decode_len().unwrap_or(0) <
					T::MaxMessagesPerBlock::get() as usize,
				Yield
			);

			if let Export(Here) = origin {
				PendingHighPriorityMessageCount::<T>::mutate(|count| {
					*count = count.saturating_sub(1)
				});
			} else {
				ensure!(!Self::operating_mode().is_halted(), Yield);
				ensure!(PendingHighPriorityMessageCount::<T>::get() == 0, Yield);
			}

			// Decode bytes into versioned message
			let versioned_enqueued_message: VersionedQueuedMessage =
				VersionedQueuedMessage::decode(&mut message).map_err(|_| Corrupt)?;

			// Convert versioned message into latest supported message version
			let enqueued_message: QueuedMessage =
				versioned_enqueued_message.try_into().map_err(|_| Unsupported)?;

			let next_nonce = Nonce::<T>::get(enqueued_message.origin).saturating_add(1);

			let command = enqueued_message.command.index();
			let params = enqueued_message.command.abi_encode();
			let max_dispatch_gas = T::GasMeter::maximum_required(&enqueued_message.command) as u128;
			let max_refund = Self::calculate_maximum_gas_refund(&enqueued_message.command);
			let reward = Self::fee_config().reward;

			// Construct the final committed message
			let message = CommittedMessage {
				origin: enqueued_message.origin,
				nonce: next_nonce,
				command,
				params,
				max_dispatch_gas,
				max_refund,
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

		/// Maximum overall gas required for delivering a message
		pub(crate) fn maximum_overall_required_gas(command: &Command) -> u64 {
			T::GasMeter::MAXIMUM_BASE_GAS + T::GasMeter::maximum_required(command)
		}

		/// Calculate fee in native currency for delivering a message.
		pub(crate) fn calculate_fee(command: &Command) -> Option<Fee<T::Balance>> {
			let max_gas = Self::maximum_overall_required_gas(command);
			let remote_fee = Self::calculate_remote_fee(
				max_gas,
				Self::fee_config().fee_per_gas,
				Self::fee_config().reward,
			);
			let remote_fee =
				FixedU128::from(remote_fee)
				.checked_div(&Self::fee_config().exchange_rate)?
				.into_inner()
				.checked_div(FixedU128::accuracy())
				.expect("accuracy is not zero; qed")
				.into();
			let local_fee = Self::calculate_local_fee();
			let fee = Fee::from((local_fee, remote_fee));

			Some(fee)
		}

		/// Calculate fee in remote currency for dispatching a message on Ethereum
		pub(crate) fn calculate_remote_fee(
			max_gas_required: u64,
			fee_per_gas: u128,
			reward: u128,
		) -> u128 {
			fee_per_gas.saturating_mul(max_gas_required.into()).saturating_add(reward)
		}

		/// Calculate fee in native currency for processing a message locally
		pub(crate) fn calculate_local_fee() -> T::Balance {
			T::WeightToFee::weight_to_fee(
				&T::WeightInfo::do_process_message().saturating_add(T::WeightInfo::commit_single()),
			)
		}

		/// Maximum refund in Ether for delivering a message
		pub(crate) fn calculate_maximum_gas_refund(command: &Command) -> u128 {
			let max_gas = Self::maximum_overall_required_gas(command);
			Self::fee_config().fee_per_gas.saturating_mul(max_gas.into())
		}
	}
}
