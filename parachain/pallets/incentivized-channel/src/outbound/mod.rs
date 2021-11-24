pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

use codec::{Decode, Encode};
use ethabi::{self, Token};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{EnsureOrigin, Get},
};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160, H256, U256};
use sp_io::offchain_index;
use sp_runtime::traits::{Hash, Zero};

use sp_std::prelude::*;

use snowbridge_core::{types::AuxiliaryDigestItem, ChannelId, MessageNonce, SingleAsset};

pub use weights::WeightInfo;

/// Wire-format for committed messages
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Message {
	/// Target application on the Ethereum side.
	target: H160,
	/// A nonce for replay protection and ordering.
	nonce: u64,
	/// Fee for accepting message on this channel.
	fee: U256,
	/// Payload for target application.
	payload: Vec<u8>,
}

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

		/// Prefix for offchain storage keys.
		const INDEXING_PREFIX: &'static [u8];

		type Hashing: Hash<Output = H256>;

		/// Max bytes in a message payload
		#[pallet::constant]
		type MaxMessagePayloadSize: Get<u64>;

		/// Max number of messages per commitment
		#[pallet::constant]
		type MaxMessagesPerCommit: Get<u64>;

		type FeeCurrency: SingleAsset<<Self as frame_system::Config>::AccountId>;

		/// The origin which may update reward related params
		type SetFeeOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		MessageAccepted(MessageNonce),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message payload exceeds byte limit.
		PayloadTooLarge,
		/// No more messages can be queued for the channel during this commit cycle.
		QueueSizeLimitReached,
		/// Cannot pay the fee to submit a message.
		NoFunds,
		/// Cannot increment nonce
		Overflow,
	}

	/// Interval between commitments
	#[pallet::storage]
	#[pallet::getter(fn interval)]
	pub(super) type Interval<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Messages waiting to be committed.
	#[pallet::storage]
	pub(super) type MessageQueue<T: Config> = StorageValue<_, Vec<Message>, ValueQuery>;

	/// Fee for accepting a message
	#[pallet::storage]
	#[pallet::getter(fn fee)]
	pub type Fee<T: Config> = StorageValue<_, U256, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub interval: T::BlockNumber,
		pub fee: U256,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { interval: Default::default(), fee: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Interval<T>>::put(self.interval);
			<Fee<T>>::put(self.fee);
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
		#[pallet::weight(T::WeightInfo::set_fee())]
		pub fn set_fee(origin: OriginFor<T>, amount: U256) -> DispatchResult {
			T::SetFeeOrigin::ensure_origin(origin)?;
			<Fee<T>>::put(amount);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Submit message on the outbound channel
		pub fn submit(who: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
			ensure!(
				<MessageQueue<T>>::decode_len().unwrap_or(0) <
					T::MaxMessagesPerCommit::get() as usize,
				Error::<T>::QueueSizeLimitReached,
			);
			ensure!(
				payload.len() <= T::MaxMessagePayloadSize::get() as usize,
				Error::<T>::PayloadTooLarge,
			);

			<Nonce<T>>::try_mutate(|nonce| -> DispatchResult {
				if let Some(v) = nonce.checked_add(1) {
					*nonce = v;
				} else {
					return Err(Error::<T>::Overflow.into())
				}

				// Attempt to charge a fee for message submission
				let fee = Self::fee();
				T::FeeCurrency::withdraw(who, fee).map_err(|_| Error::<T>::NoFunds)?;

				<MessageQueue<T>>::append(Message {
					target,
					nonce: *nonce,
					fee,
					payload: payload.to_vec(),
				});
				Self::deposit_event(Event::MessageAccepted(*nonce));
				Ok(())
			})
		}

		fn commit() -> Weight {
			let messages: Vec<Message> = <MessageQueue<T>>::take();
			if messages.is_empty() {
				return T::WeightInfo::on_initialize_no_messages()
			}

			let commitment_hash = Self::make_commitment_hash(&messages);
			let average_payload_size = Self::average_payload_size(&messages);

			let digest_item =
				AuxiliaryDigestItem::Commitment(ChannelId::Incentivized, commitment_hash.clone())
					.into();
			<frame_system::Pallet<T>>::deposit_log(digest_item);

			let key = Self::make_offchain_key(commitment_hash);
			offchain_index::set(&*key, &messages.encode());

			T::WeightInfo::on_initialize(messages.len() as u32, average_payload_size as u32)
		}

		fn make_commitment_hash(messages: &[Message]) -> H256 {
			let messages: Vec<Token> = messages
				.iter()
				.map(|message| {
					Token::Tuple(vec![
						Token::Address(message.target),
						Token::Uint(message.nonce.into()),
						Token::Uint(message.fee.into()),
						Token::Bytes(message.payload.clone()),
					])
				})
				.collect();
			let input = ethabi::encode(&vec![Token::Array(messages)]);
			<T as Config>::Hashing::hash(&input)
		}

		fn average_payload_size(messages: &[Message]) -> usize {
			let sum: usize = messages.iter().fold(0, |acc, x| acc + x.payload.len());
			// We overestimate message payload size rather than underestimate.
			// So add 1 here to account for integer division truncation.
			(sum / messages.len()).saturating_add(1)
		}

		fn make_offchain_key(hash: H256) -> Vec<u8> {
			(T::INDEXING_PREFIX, ChannelId::Incentivized, hash).encode()
		}
	}
}
