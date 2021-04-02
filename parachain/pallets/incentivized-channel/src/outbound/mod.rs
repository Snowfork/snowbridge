use codec::{Encode, Decode};
use ethabi::{self, Token};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	weights::Weight,
	traits::Get,
	dispatch::DispatchResult,
};
use frame_system::{self as system};
use sp_core::{H160, H256, RuntimeDebug};
use sp_io::offchain_index;
use sp_runtime::{
	traits::{Hash, Zero},
};
use sp_std::prelude::*;

use artemis_core::{ChannelId, MessageNonce, types::AuxiliaryDigestItem};

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

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Prefix for offchain storage keys.
	const INDEXING_PREFIX: &'static [u8];

	type Hashing: Hash<Output = H256>;

	/// Max number of messages that can be queued and committed in one go
	type MaxMessagesPerCommit: Get<usize>;
}

decl_storage! {
	trait Store for Module<T: Config> as IncentivizedOutboundModule {
		/// Interval between committing messages.
		Interval get(fn interval) config(): T::BlockNumber;

		/// Messages waiting to be committed.
		MessageQueue get(fn incentivized_mq): Vec<Message>;

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
		/// No more messages can be queued for the channel during this commit cycle.
		QueueSizeLimitReached,
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
				Self::commit_messages()
			} else {
				0
			}
		}
	}
}

pub fn offchain_key(prefix: &[u8], hash: H256) -> Vec<u8> {
	(prefix, ChannelId::Incentivized, hash).encode()
}

impl<T: Config> Module<T> {
	pub fn submit(_: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
		Nonce::try_mutate(|nonce| -> DispatchResult {
			*nonce += 1;
			Self::push_message(target, *nonce, payload)?;
			<Module<T>>::deposit_event(Event::MessageAccepted(*nonce));
			Ok(())
		})
	}

	// Add a message for eventual inclusion in a commitment
	// TODO (Security): Limit number of messages per commitment
	//   https://github.com/Snowfork/polkadot-ethereum/issues/226
	fn push_message(target: H160, nonce: u64, payload: &[u8]) -> DispatchResult {
		ensure!(
			MessageQueue::decode_len().unwrap_or(0) < T::MaxMessagesPerCommit::get(),
			Error::<T>::QueueSizeLimitReached,
		);

		let mut mq = MessageQueue::get();
		mq.push(Message {
				target,
				nonce,
				payload: payload.to_vec(),
			});
		MessageQueue::put(mq);
		Ok(())
	}

	// TODO: return proper weight
	fn commit_messages() -> Weight {
		let messages: Vec<Message> = <Self as Store>::MessageQueue::get();
		if messages.is_empty() {
			return 0
		}

		let commitment = Self::encode_commitment(&messages);
		let commitment_hash = <T as Config>::Hashing::hash(&commitment);

		let digest_item = AuxiliaryDigestItem::Commitment(ChannelId::Incentivized, commitment_hash.clone()).into();
		<frame_system::Module<T>>::deposit_log(digest_item);

		let key = offchain_key(T::INDEXING_PREFIX, commitment_hash);
		offchain_index::set(&*key, &messages.encode());

		// Clear queue
		<Self as Store>::MessageQueue::put(<Vec<Message>>::new());

		0
	}

	// ABI-encode the commitment
	fn encode_commitment(commitment: &[Message]) -> Vec<u8> {
		let messages: Vec<Token> = commitment
			.iter()
			.map(|message|
				Token::Tuple(vec![
					Token::Address(message.target),
					Token::Uint(message.nonce.into()),
					Token::Bytes(message.payload.clone())
				])
			)
			.collect();
		ethabi::encode(&vec![Token::Array(messages)])
	}
}
