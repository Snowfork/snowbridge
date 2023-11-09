//! Implementation for [`snowbridge_core::outbound::SendMessage`]
use super::*;
use codec::Encode;
use frame_support::{
	ensure,
	traits::{EnqueueMessage, Get},
	CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use snowbridge_core::outbound::{
	AggregateMessageOrigin, Fee, Message, QueuedMessage, SendError, SendMessage,
	SnowbridgeMessageOrigin, VersionedQueuedMessage,
};
use sp_core::H256;
use sp_runtime::BoundedVec;

/// The maximal length of an enqueued message, as determined by the MessageQueue pallet
pub type MaxEnqueuedMessageSizeOf<T> =
	<<T as Config>::MessageQueue as EnqueueMessage<AggregateMessageOrigin>>::MaxMessageLen;

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound)]
pub struct Ticket<MaxMessageSize: Get<u32>> {
	pub id: H256,
	pub origin: ParaId,
	pub message: BoundedVec<u8, MaxMessageSize>,
}

impl<T: Config> SendMessage for Pallet<T> {
	type Ticket = Ticket<MaxEnqueuedMessageSizeOf<T>>;
	type Balance = T::Balance;

	fn validate(message: &Message) -> Result<(Self::Ticket, Fee<Self::Balance>), SendError> {
		// The inner payload should not be too large
		let payload = message.command.abi_encode();

		// Create a message id for tracking progress in submission pipeline
		let message_id: H256 = sp_io::hashing::blake2_256(&(message.encode())).into();

		ensure!(
			payload.len() < T::MaxMessagePayloadSize::get() as usize,
			SendError::MessageTooLarge
		);

		let fee = Self::calculate_fee(&message.command);

		let queued_message: VersionedQueuedMessage = QueuedMessage {
			id: message_id,
			origin: message.origin,
			command: message.command.clone(),
		}
		.into();
		// The whole message should not be too large
		let encoded = queued_message.encode().try_into().map_err(|_| SendError::MessageTooLarge)?;

		let ticket = Ticket { id: message_id, origin: message.origin, message: encoded };

		Ok((ticket, fee))
	}

	fn deliver(ticket: Self::Ticket) -> Result<H256, SendError> {
		use AggregateMessageOrigin::*;
		use SnowbridgeMessageOrigin::*;

		// Assign an `AggregateMessageOrigin` to track the message within the MessageQueue
		// pallet. Governance commands are assigned origin `ExportOrigin::Here`. In other words
		// emitted from BridgeHub itself.
		let origin = if ticket.origin == T::OwnParaId::get() {
			Snowbridge(Here)
		} else {
			Snowbridge(Sibling(ticket.origin))
		};

		if let Snowbridge(Here) = origin {
			// Increase PendingHighPriorityMessageCount by one
			PendingHighPriorityMessageCount::<T>::mutate(|count| *count = count.saturating_add(1));
		} else {
			ensure!(!Self::operating_mode().is_halted(), SendError::Halted);
		}

		let message = ticket.message.as_bounded_slice();

		T::MessageQueue::enqueue_message(message, origin);
		Self::deposit_event(Event::MessageQueued { id: ticket.id });
		Ok(ticket.id)
	}
}
