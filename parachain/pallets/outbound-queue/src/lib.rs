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
#![cfg_attr(not(feature = "std"), no_std)]
pub mod api;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use codec::{Decode, Encode};
use ethabi::Token;
use frame_support::{
	ensure,
	storage::StorageStreamIter,
	traits::{tokens::Balance, EnqueueMessage, Get, ProcessMessage, ProcessMessageError},
	weights::{Weight, WeightToFee},
	CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use scale_info::TypeInfo;
use snowbridge_core::ParaId;
use sp_arithmetic::FixedU128;
use sp_core::{bounded_vec::BoundedVec, H256};
use sp_runtime::{traits::Hash, FixedPointNumber};

use sp_std::prelude::*;

use snowbridge_core::{
	outbound::{
		AggregateMessageOrigin, Command, EnqueuedMessage, ExportOrigin, Fees, GasMeter, Message,
		OutboundQueue as OutboundQueueTrait, SendError, VersionedEnqueuedMessage,
	},
	BasicOperatingMode,
};
use snowbridge_outbound_queue_merkle_tree::merkle_root;
pub use snowbridge_outbound_queue_merkle_tree::MerkleProof;
pub use weights::WeightInfo;

/// The maximal length of an enqueued message, as determined by the MessageQueue pallet
pub type MaxEnqueuedMessageSizeOf<T> =
	<<T as Config>::MessageQueue as EnqueueMessage<AggregateMessageOrigin>>::MaxMessageLen;

pub type ProcessMessageOriginOf<T> = <Pallet<T> as ProcessMessage>::Origin;

pub use pallet::*;

pub const LOG_TARGET: &str = "snowbridge-outbound-queue";

/// A message which can be accepted by the [`OutboundQueue`]
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound)]
pub struct OutboundQueueTicket<MaxMessageSize: Get<u32>> {
	pub id: H256,
	pub origin: ParaId,
	pub message: BoundedVec<u8, MaxMessageSize>,
}

/// Message which has been assigned a nonce and will be committed at the end of a block
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
pub struct PreparedMessage {
	/// ID of source parachain
	pub origin: ParaId,
	/// Unique nonce to prevent replaying messages
	pub nonce: u64,
	/// Command to execute in the Gateway contract
	pub command: u8,
	/// Params for the command
	pub params: Vec<u8>,
	/// Maximum gas allowed for message dispatch
	pub max_dispatch_gas: u128,
	/// Maximum gas refund for message relayer
	pub max_refund: u128,
	/// Reward in ether for delivering this message, in addition to the gas refund
	pub reward: u128,
}

/// Convert message into an ABI-encoded form for delivery to the InboundQueue contract on Ethereum
impl From<PreparedMessage> for Token {
	fn from(x: PreparedMessage) -> Token {
		Token::Tuple(vec![
			Token::Uint(u32::from(x.origin).into()),
			Token::Uint(x.nonce.into()),
			Token::Uint(x.command.into()),
			Token::Bytes(x.params.to_vec()),
			Token::Uint(x.max_dispatch_gas.into()),
			Token::Uint(x.max_refund.into()),
			Token::Uint(x.reward.into()),
		])
	}
}

pub const GWEI: u128 = 1_000_000_000;
pub const ETH: u128 = 1_000_000_000_000_000_000;

pub const MAX_FEE_PER_GAS: u128 = 300 * GWEI;
pub const MAX_REWARD: u128 = 1 * ETH;
pub const MAX_EXCHANGE_RATE: FixedU128 = FixedU128::from_rational(1000, 1);

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
		OperatingModeChanged { mode: BasicOperatingMode },
		ExchangeRateChanged {
			/// ETH/DOT exchange rate
			exchange_rate: FixedU128,
		},
		FeePerGasChanged {
			/// Ether fee per unit of gas consumed on Ethereum
			fee_per_gas: u128,
		},
		RewardChanged {
			/// Ether reward paid to relayers for delivering a message to Ethereum
			reward: u128,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message is too large
		MessageTooLarge,
		/// The pallet is halted
		Halted,
		/// Invalid fee param
		InvalidFeeParam,
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

	/// ETH/DOT exchange rate
	#[pallet::storage]
	#[pallet::getter(fn exchange_rate)]
	pub type ExchangeRate<T: Config> = StorageValue<_, FixedU128, ValueQuery>;

	/// Ether fee per unit of gas consumed on Ethereum
	#[pallet::storage]
	#[pallet::getter(fn fee_per_gas)]
	pub type FeePerGas<T: Config> = StorageValue<_, u128, ValueQuery>;

	/// Ether reward for delivering message to Ethereum
	#[pallet::storage]
	#[pallet::getter(fn reward)]
	pub type Reward<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		#[serde(skip)]
		pub phantom: PhantomData<T>,
		pub operating_mode: BasicOperatingMode,
		pub exchange_rate: FixedU128,
		pub fee_per_gas: u128,
		pub reward: u128,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			OperatingMode::<T>::put(self.operating_mode);
			ExchangeRate::<T>::put(self.exchange_rate);
			FeePerGas::<T>::put(self.fee_per_gas);
			Reward::<T>::put(self.reward);
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
			T::WeightInfo::commit_messages()
		}

		fn on_finalize(_: BlockNumberFor<T>) {
			Self::commit_messages();
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

		#[pallet::call_index(1)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_exchange_rate(origin: OriginFor<T>, exchange_rate: FixedU128) -> DispatchResult {
			ensure_root(origin)?;
			// An extra sanity check to ensure that governance doesn't brick the bridge
			// by mistakenly configuring a huge fee
			if exchange_rate > MAX_EXCHANGE_RATE {
				return Err(Error::<T>::InvalidFeeParam.into())
			}
			ExchangeRate::<T>::put(exchange_rate);
			Self::deposit_event(Event::ExchangeRateChanged { exchange_rate });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_fee_per_gas(origin: OriginFor<T>, fee_per_gas: u128) -> DispatchResult {
			ensure_root(origin)?;
			// An extra sanity check to ensure that governance doesn't brick the bridge
			// by mistakenly configuring a huge fee
			if fee_per_gas > MAX_FEE_PER_GAS {
				return Err(Error::<T>::InvalidFeeParam.into())
			}
			FeePerGas::<T>::put(fee_per_gas);
			Self::deposit_event(Event::FeePerGasChanged { fee_per_gas });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_reward(origin: OriginFor<T>, reward: u128) -> DispatchResult {
			ensure_root(origin)?;
			// An extra sanity check to ensure that governance doesn't brick the bridge
			// by mistakenly configuring a huge fee
			if reward > MAX_REWARD {
				return Err(Error::<T>::InvalidFeeParam.into())
			}
			Reward::<T>::put(reward);
			Self::deposit_event(Event::RewardChanged { reward });
			Ok(())
		}
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
		pub(crate) fn do_process_message(
			origin: ProcessMessageOriginOf<T>,
			mut message: &[u8],
		) -> Result<bool, ProcessMessageError> {
			// Yield if the maximum number of messages has been processed this block.
			// This ensures that the weight of `on_finalize` has a known maximum bound.
			ensure!(
				MessageLeaves::<T>::decode_len().unwrap_or(0) <
					T::MaxMessagesPerBlock::get() as usize,
				ProcessMessageError::Yield
			);

			if let AggregateMessageOrigin::Export(ExportOrigin::Here) = origin {
				// Decrease PendingHighPriorityMessageCount by one
				PendingHighPriorityMessageCount::<T>::mutate(|count| {
					*count = count.saturating_sub(1)
				});
			} else {
				ensure!(!Self::operating_mode().is_halted(), ProcessMessageError::Yield);
				ensure!(
					PendingHighPriorityMessageCount::<T>::get() == 0,
					ProcessMessageError::Yield
				);
			}

			// Decode bytes into versioned message
			let versioned_enqueued_message: VersionedEnqueuedMessage =
				VersionedEnqueuedMessage::decode(&mut message)
					.map_err(|_| ProcessMessageError::Corrupt)?;

			// Convert versioned message into latest supported message version
			let enqueued_message: EnqueuedMessage = versioned_enqueued_message
				.try_into()
				.map_err(|_| ProcessMessageError::Unsupported)?;

			let next_nonce = Nonce::<T>::get(enqueued_message.origin).saturating_add(1);

			let command = enqueued_message.command.index();
			let params = enqueued_message.command.abi_encode();
			let max_dispatch_gas = T::GasMeter::maximum_required(&enqueued_message.command) as u128;
			let max_refund = Self::maximum_refund(&enqueued_message.command);
			let reward = Self::reward();

			// Construct a prepared message, which when ABI-encoded is what the
			// other side of the bridge will verify.
			let message: PreparedMessage = PreparedMessage {
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

		/// Fee in DOT for delivering a message.
		pub(crate) fn calculate_delivery_fee(command: &Command) -> T::Balance {
			// Calculate maximum gas that can be consumed by a message
			let max_gas = Self::maximum_overall_required_gas(command);

			// Calculate fee in Ether (gas cost + additional delivery reward)
			let fee = Self::fee_per_gas()
				.saturating_mul(max_gas.into())
				.saturating_add(Self::reward());

			// Convert to native token (DOT or KSM)
			(Self::exchange_rate() * fee.into())
				.into_inner()
				.checked_div(FixedU128::accuracy())
				.expect("checked in prep, qed")
				.into()
		}

		/// Maximum refund in Ether for delivering a message
		pub(crate) fn maximum_refund(command: &Command) -> u128 {
			let max_gas = Self::maximum_overall_required_gas(command);
			Self::fee_per_gas().saturating_mul(max_gas.into())
		}
	}

	impl<T: Config> OutboundQueueTrait for Pallet<T> {
		type Ticket = OutboundQueueTicket<MaxEnqueuedMessageSizeOf<T>>;
		type Balance = T::Balance;

		fn validate(message: &Message) -> Result<(Self::Ticket, Fees<Self::Balance>), SendError> {
			// The inner payload should not be too large
			let payload = message.command.abi_encode();

			// Create a message id for tracking progress in submission pipeline
			let message_id: H256 = sp_io::hashing::blake2_256(&(message.encode())).into();

			ensure!(
				payload.len() < T::MaxMessagePayloadSize::get() as usize,
				SendError::MessageTooLarge
			);

			// calculate processing fees which include a local and remote component
			let fee = Fees {
				base: T::WeightToFee::weight_to_fee(
					&T::WeightInfo::do_process_message()
						.saturating_add(T::WeightInfo::commit_one_message()),
				),
				delivery: Self::calculate_delivery_fee(&message.command),
			};

			let enqueued_message: VersionedEnqueuedMessage = EnqueuedMessage {
				id: message_id,
				origin: message.origin,
				command: message.command.clone(),
			}
			.into();
			// The whole message should not be too large
			let encoded =
				enqueued_message.encode().try_into().map_err(|_| SendError::MessageTooLarge)?;

			let ticket =
				OutboundQueueTicket { id: message_id, origin: message.origin, message: encoded };

			Ok((ticket, fee))
		}

		fn submit(ticket: Self::Ticket) -> Result<H256, SendError> {
			// Assign an `AggregateMessageOrigin` to track the message within the MessageQueue
			// pallet. Governance commands are assigned origin `ExportOrigin::Here`. In other words
			// emitted from BridgeHub itself.
			let origin = if ticket.origin == T::OwnParaId::get() {
				AggregateMessageOrigin::Export(ExportOrigin::Here)
			} else {
				AggregateMessageOrigin::Export(ExportOrigin::Sibling(ticket.origin))
			};

			if let AggregateMessageOrigin::Export(ExportOrigin::Here) = origin {
				// Increase PendingHighPriorityMessageCount by one
				PendingHighPriorityMessageCount::<T>::mutate(|count| {
					*count = count.saturating_add(1)
				});
			} else {
				ensure!(!Self::operating_mode().is_halted(), SendError::Halted);
			}

			T::MessageQueue::enqueue_message(ticket.message.as_bounded_slice(), origin);
			Self::deposit_event(Event::MessageQueued { id: ticket.id });
			Ok(ticket.id)
		}
	}

	impl<T: Config> ProcessMessage for Pallet<T> {
		type Origin = AggregateMessageOrigin;
		fn process_message(
			message: &[u8],
			origin: Self::Origin,
			meter: &mut frame_support::weights::WeightMeter,
			_: &mut [u8; 32],
		) -> Result<bool, ProcessMessageError> {
			let weight = T::WeightInfo::do_process_message();
			if !meter.check_accrue(weight) {
				return Err(ProcessMessageError::Overweight(weight))
			}
			Self::do_process_message(origin, message)
		}
	}
}
