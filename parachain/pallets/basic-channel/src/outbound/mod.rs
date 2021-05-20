use codec::{Encode, Decode};
use ethabi::{self, Token};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	weights::Weight,
	dispatch::DispatchResult,
	traits::{Get, EnsureOrigin},
	ensure,
};
use frame_system::{self as system};
use sp_core::{H160, H256, RuntimeDebug};
use sp_io::offchain_index;
use sp_runtime::{
	traits::{Hash, Zero, StaticLookup},
};
use sp_std::prelude::*;

use artemis_core::{ChannelId, MessageNonce, types::AuxiliaryDigestItem};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod test;

/// Wire-format for committed messages
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub struct Message {
	/// Target application on the Ethereum side.
	target: H160,
	/// A nonce for replay protection and ordering.
	nonce: u64,
	/// Payload for target application.
	payload: Vec<u8>,
}

/// Weight functions needed for this pallet.
pub trait WeightInfo {
	fn on_initialize(num_messages: u32, avg_payload_bytes: u32) -> Weight;
	fn on_initialize_non_interval() -> Weight;
	fn on_initialize_no_messages() -> Weight;
	fn set_principal() -> Weight;
}

impl WeightInfo for () {
	fn on_initialize(_: u32, _: u32) -> Weight { 0 }
	fn on_initialize_non_interval() -> Weight { 0 }
	fn on_initialize_no_messages() -> Weight { 0 }
	fn set_principal() -> Weight { 0 }
}

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Prefix for offchain storage keys.
	const INDEXING_PREFIX: &'static [u8];

	type Hashing: Hash<Output = H256>;

	// Max bytes in a message payload
	type MaxMessagePayloadSize: Get<usize>;

	/// Max number of messages that can be queued and committed in one go for a given channel.
	type MaxMessagesPerCommit: Get<usize>;

	type SetPrincipalOrigin: EnsureOrigin<Self::Origin>;

	/// Weight information for extrinsics in this pallet
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as BasicOutboundModule {
		/// Interval between committing messages.
		Interval get(fn interval) config(): T::BlockNumber;

		/// Messages waiting to be committed.
		MessageQueue: Vec<Message>;

		/// The Account authorized to submit messages
		Principal get(fn principal) config(): T::AccountId;

		pub Nonce: u64;
	}
}

decl_event! {
	pub enum Event {
		MessageAccepted(MessageNonce),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// The message payload exceeds byte limit.
		PayloadTooLarge,
		/// No more messages can be queued for the channel during this commit cycle.
		QueueSizeLimitReached,
		/// Cannot increment nonce
		Overflow,
		/// Not authorized to send message
		NotAuthorized,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

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

		#[weight = T::WeightInfo::set_principal()]
		fn set_principal(origin, principal: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
			T::SetPrincipalOrigin::ensure_origin(origin)?;
			let principal = T::Lookup::lookup(principal)?;
			<Principal<T>>::put(principal);
			Ok(())
		}
	}
}

impl<T: Config> Module<T> {

	/// Submit message on the outbound channel
	pub fn submit(who: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
		ensure!(
			*who == Self::principal(),
			Error::<T>::NotAuthorized,
		);
		ensure!(
			MessageQueue::decode_len().unwrap_or(0) < T::MaxMessagesPerCommit::get(),
			Error::<T>::QueueSizeLimitReached,
		);
		ensure!(
			payload.len() <= T::MaxMessagePayloadSize::get(),
			Error::<T>::PayloadTooLarge,
		);

		Nonce::try_mutate(|nonce| -> DispatchResult {
			if let Some(v) = nonce.checked_add(1) {
				*nonce = v;
			} else {
				return Err(Error::<T>::Overflow.into())
			}

			MessageQueue::append(
				Message {
					target,
					nonce: *nonce,
					payload: payload.to_vec(),
				},
			);
			<Module<T>>::deposit_event(Event::MessageAccepted(*nonce));
			Ok(())
		})
	}

	fn commit() -> Weight {
		let messages: Vec<Message> = MessageQueue::take();
		if messages.is_empty() {
			return T::WeightInfo::on_initialize_no_messages();
		}

		let commitment_hash = Self::make_commitment_hash(&messages);
		let average_payload_size = Self::average_payload_size(&messages);

		let digest_item = AuxiliaryDigestItem::Commitment(
			ChannelId::Incentivized,
			commitment_hash.clone()
		).into();
		<frame_system::Pallet<T>>::deposit_log(digest_item);

		let key = Self::make_offchain_key(commitment_hash);
		offchain_index::set(&*key, &messages.encode());

		T::WeightInfo::on_initialize(
			messages.len() as u32,
			average_payload_size as u32
		)
	}

	fn make_commitment_hash(messages: &[Message]) -> H256 {
		let mut payload_size: usize = 0;
		let messages: Vec<Token> = messages
			.iter()
			.map(|message| {
				payload_size += message.payload.len();
				Token::Tuple(vec![
					Token::Address(message.target),
					Token::Uint(message.nonce.into()),
					Token::Bytes(message.payload.clone())
				])
			})
			.collect();
		let input = ethabi::encode(&vec![Token::Array(messages)]);
		<T as Config>::Hashing::hash(&input)
	}

	fn average_payload_size(messages: &[Message]) -> usize {
		let sum: usize = messages.iter()
			.fold(0, |acc, x| acc + x.payload.len());
		// We overestimate message payload size rather than underestimate.
		// So add 1 here to account for integer division truncation.
		(sum / messages.len()).saturating_add(1)
	}

	fn make_offchain_key(hash: H256) -> Vec<u8> {
		(T::INDEXING_PREFIX, ChannelId::Basic, hash).encode()
	}
}
