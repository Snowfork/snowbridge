//! Implementation for [`frame_support::traits::QueuePausedQuery`]
use super::*;
use frame_support::traits::QueuePausedQuery;
use snowbridge_core::outbound::AggregateMessageOrigin;

impl<T> QueuePausedQuery<AggregateMessageOrigin> for Pallet<T>
where
	T: Config,
{
	fn is_paused(origin: &AggregateMessageOrigin) -> bool {
		use AggregateMessageOrigin::*;

		// Queues for sibling parachains are paused when:
		// 1. The pallet is halted
		// 2. A higher-priority queue has pending messages
		match origin {
			Snowbridge(channel_id) if *channel_id != T::GovernanceChannelId::get() =>
				if Self::operating_mode().is_halted() {
					true
				} else if PendingHighPriorityMessageCount::<T>::get() > 0 {
					true
				} else {
					false
				},
			_ => false,
		}
	}
}
