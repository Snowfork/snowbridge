pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

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

pub use weights::WeightInfo;

/// Wire-format for committed messages
#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, MaxEncodedLen, TypeInfo,
)]
#[scale_info(skip_type_params(M, N))]
#[codec(mel_bound())]
pub struct MessageBundle<M: Get<u32>, N: Get<u32>> {
	source_channel_id: u8,
	/// Unique nonce for to prevent replaying bundles
	#[codec(compact)]
	nonce: u64,
	messages: BoundedVec<Message<M>, N>,
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

pub type MessageBundleOf<T> =
	MessageBundle<<T as Config>::MaxMessagePayloadSize, <T as Config>::MaxMessagesPerCommit>;
pub type MessageOf<T> = Message<<T as Config>::MaxMessagePayloadSize>;

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
		StorageValue<_, BoundedVec<MessageOf<T>, T::MaxMessagesPerCommit>, ValueQuery>;

	/// Fee for accepting a message
	#[pallet::storage]
	#[pallet::getter(fn principal)]
	pub type Principal<T: Config> = StorageValue<_, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type NextId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub interval: T::BlockNumber,
		pub principal: Option<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { interval: Default::default(), principal: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Interval<T>>::put(self.interval);
			<Principal<T>>::put(self.principal.clone());
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
			T::SetPrincipalOrigin::ensure_origin(origin)?;
			let principal = T::Lookup::lookup(principal)?;
			<Principal<T>>::put(Some(principal));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Submit message on the outbound channel
		pub fn submit(who: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
			let principal = Self::principal();
			ensure!(principal.is_some(), Error::<T>::NotAuthorized,);
			ensure!(*who == principal.unwrap(), Error::<T>::NotAuthorized,);
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

			<MessageQueue<T>>::try_append(Message {
				id: next_id,
				target,
				payload: payload.to_vec().try_into().map_err(|_| Error::<T>::PayloadTooLarge)?,
			})
			.map_err(|_| Error::<T>::QueueSizeLimitReached)?;
			Self::deposit_event(Event::MessageAccepted(next_id));

			<NextId<T>>::put(next_id + 1);

			Ok(())
		}

		fn commit() -> Weight {
			let messages = <MessageQueue<T>>::take();
			if messages.is_empty() {
				return T::WeightInfo::on_initialize_no_messages();
			}

			let nonce = <Nonce<T>>::get();
			let next_nonce = nonce.saturating_add(1);
			<Nonce<T>>::put(next_nonce);

			let bundle = MessageBundle {
				source_channel_id: ChannelId::Basic as u8,
				nonce: next_nonce,
				messages: messages.clone(),
			};

			let commitment_hash = Self::make_commitment_hash(&bundle);
			let digest_item =
				AuxiliaryDigestItem::Commitment(ChannelId::Basic, commitment_hash.clone()).into();
			<frame_system::Pallet<T>>::deposit_log(digest_item);
			Self::deposit_event(Event::Committed { hash: commitment_hash, data: bundle });

			T::WeightInfo::on_initialize(
				messages.len() as u32,
				Self::average_payload_size(&messages),
			)
		}

		fn make_commitment_hash(bundle: &MessageBundleOf<T>) -> H256 {
			let messages: Vec<Token> = bundle
				.messages
				.iter()
				.map(|message| {
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

		fn average_payload_size(messages: &[MessageOf<T>]) -> u32 {
			let sum: usize = messages.iter().fold(0, |acc, x| acc + x.payload.len());
			// We overestimate message payload size rather than underestimate.
			// So add 1 here to account for integer division truncation.
			(sum / messages.len()).saturating_add(1) as u32
		}
	}
}
