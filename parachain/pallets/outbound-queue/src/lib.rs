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
use xcm::prelude::{MultiAsset, MultiAssets, MultiLocation};

use snowbridge_core::outbound::{
	Command, FeeAmount, GasAmount, Message, MessageHash, OutboundFeeConfig,
	OutboundQueue as OutboundQueueTrait, OutboundQueueTicket, SubmitError,
};
use snowbridge_outbound_queue_merkle_tree::merkle_root;

pub use snowbridge_outbound_queue_merkle_tree::MerkleProof;
use sp_runtime::{FixedU128, Saturating};
pub use weights::WeightInfo;

/// Aggregate message origin for the `MessageQueue` pallet.
#[derive(Encode, Decode, Clone, MaxEncodedLen, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AggregateMessageOrigin {
	#[codec(index = 0)]
	Parachain(ParaId),
}

/// Message which is awaiting processing in the MessageQueue pallet
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct EnqueuedMessage {
	/// Message ID (usually hash of message)
	pub id: H256,
	/// ID of source parachain
	pub origin: ParaId,
	/// Command to execute in the Gateway contract
	pub command: Command,
}

/// Message which has been assigned a nonce and will be committed at the end of a block
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
pub struct PreparedMessage {
	/// ID of source parachain
	origin: ParaId,
	/// Unique nonce to prevent replaying messages
	nonce: u64,
	/// Command to execute in the Gateway contract
	command: u8,
	/// Params for the command
	params: Vec<u8>,
	/// Maximum gas allowed for message dispatch
	dispatch_gas: u128,
	/// Reward in ether for delivering this message
	reward: u128,
}

/// Convert message into an ABI-encoded form for delivery to the InboundQueue contract on Ethereum
impl From<PreparedMessage> for Token {
	fn from(x: PreparedMessage) -> Token {
		Token::Tuple(vec![
			Token::Uint(u32::from(x.origin).into()),
			Token::Uint(x.nonce.into()),
			Token::Uint(x.command.into()),
			Token::Bytes(x.params.to_vec()),
			Token::Uint(x.dispatch_gas.into()),
			Token::Uint(x.reward.into()),
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
		/// Set outbound fee config
		OutboundFeeConfigSet { config: OutboundFeeConfig },
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

	/// Fee config for outbound message
	#[pallet::type_value]
	pub fn DefaultFeeConfig() -> OutboundFeeConfig {
		OutboundFeeConfig::default()
	}

	#[pallet::storage]
	pub type FeeConfig<T: Config> =
		StorageValue<_, OutboundFeeConfig, ValueQuery, DefaultFeeConfig>;

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

		/// Set fee config for outbound message.
		/// May only be called by root
		#[pallet::call_index(2)]
		#[pallet::weight((T::DbWeight::get().reads_writes(2, 3), DispatchClass::Operational))]
		pub fn set_outbound_fee_config(
			origin: OriginFor<T>,
			config: OutboundFeeConfig,
		) -> DispatchResult {
			ensure_root(origin)?;
			let mut current = FeeConfig::<T>::get();
			if config.base_fee.is_some() {
				current.base_fee = config.base_fee;
			}
			if config.command_gas_map.is_some() {
				current.command_gas_map = config.command_gas_map;
			}
			if config.gas_range.is_some() {
				current.gas_range = config.gas_range;
			}
			if config.gas_price.is_some() {
				current.gas_price = config.gas_price;
			}
			if config.swap_ratio.is_some() {
				current.swap_ratio = config.swap_ratio;
			}
			if config.reward_ratio.is_some() {
				current.reward_ratio = config.reward_ratio;
			}
			FeeConfig::<T>::put(current.clone());
			Self::deposit_event(Event::OutboundFeeConfigSet { config: current });
			Ok(())
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

			let (command, params, dispatch_gas) = enqueued_message.command.abi_encode();

			let fee_config = FeeConfig::<T>::get();

			let reward = fee_config
				.reward_ratio
				.and_then(|ratio| {
					Some(ratio * dispatch_gas * fee_config.gas_price.unwrap_or_default())
				})
				.unwrap_or_default();

			// Construct a prepared message, which when ABI-encoded is what the
			// other side of the bridge will verify.
			let message: PreparedMessage = PreparedMessage {
				origin: enqueued_message.origin,
				nonce: next_nonce,
				command,
				params,
				dispatch_gas,
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

		// Todo: for arbitrary transact dispatch_gas should be dynamic retrieved from input
		pub fn get_dispatch_gas(message: &Message) -> Result<GasAmount, SubmitError> {
			let fee_config = FeeConfig::<T>::get();
			let dispatch_gas = match fee_config.command_gas_map {
				Some(command_gas_map) => *command_gas_map
					.get(&message.command.index())
					.unwrap_or(&message.command.dispatch_gas()),
				None => message.command.dispatch_gas(),
			};
			if fee_config.gas_range.is_some() {
				ensure!(
					dispatch_gas >= fee_config.gas_range.clone().unwrap_or_default().min &&
						dispatch_gas <= fee_config.gas_range.clone().unwrap_or_default().max,
					SubmitError::InvalidGas(dispatch_gas)
				);
			}
			Ok(dispatch_gas)
		}

		pub fn estimate_extra_fee(message: &Message) -> Result<Option<FeeAmount>, SubmitError> {
			let fee_config = FeeConfig::<T>::get();
			let extra_fee = match message.command.extra_fee_required() {
				true => {
					let dispatch_gas = Self::get_dispatch_gas(message)?;
					let gas_cost_in_wei =
						dispatch_gas.saturating_mul(fee_config.gas_price.unwrap_or_default());
					let gas_cost_in_native = FixedU128::from_inner(gas_cost_in_wei)
						.saturating_mul(fee_config.swap_ratio.unwrap_or_default());
					Some(gas_cost_in_native.into_inner())
				},
				false => None,
			};
			Ok(extra_fee)
		}

		/// base fee to cover the cost in bridgeHub assuming with congestion into consideration it's
		/// not a static value so load from storage configurable
		pub fn estimate_base_fee(message: &Message) -> Result<Option<FeeAmount>, SubmitError> {
			let fee_config = FeeConfig::<T>::get();
			let base_fee = match message.command.base_fee_required() {
				true => Some(fee_config.base_fee.unwrap_or_default()),
				false => None,
			};
			Ok(base_fee)
		}
	}

	impl<T: Config> OutboundQueueTrait for Pallet<T> {
		type Ticket = OutboundQueueTicket<MaxEnqueuedMessageSizeOf<T>>;

		fn validate(message: &Message) -> Result<Self::Ticket, SubmitError> {
			// The inner payload should not be too large
			let (_, payload, _) = message.command.abi_encode();

			// Create a message id for tracking progress in submission pipeline
			let message_id: MessageHash = sp_io::hashing::blake2_256(&(message.encode())).into();

			ensure!(
				payload.len() < T::MaxMessagePayloadSize::get() as usize,
				SubmitError::MessageTooLarge
			);
			let command = message.command.clone();
			let enqueued_message: EnqueuedMessage = EnqueuedMessage {
				id: message_id,
				origin: message.origin,
				command: command.clone(),
			};
			// The whole message should not be too large
			let encoded =
				enqueued_message.encode().try_into().map_err(|_| SubmitError::MessageTooLarge)?;

			let ticket = OutboundQueueTicket {
				id: message_id,
				origin: message.origin,
				message: encoded,
				command,
			};
			Ok(ticket)
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

		fn estimate_fee(message: &Message) -> Result<MultiAssets, SubmitError> {
			let base_fee = Self::estimate_base_fee(message)?.unwrap_or(FeeAmount::default());
			let extra_fee = Self::estimate_extra_fee(message)?.unwrap_or(FeeAmount::default());
			Ok(MultiAssets::from(vec![MultiAsset::from((
				MultiLocation::parent(),
				base_fee.saturating_add(extra_fee),
			))]))
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
