//! Implementation for [`frame_support::traits::QueuePausedQuery`]
use super::*;
use bridge_hub_common::{AggregateMessageOrigin, SnowbridgeMessageOrigin::Sibling};
use frame_support::traits::QueuePausedQuery;

impl<T> QueuePausedQuery<AggregateMessageOrigin> for Pallet<T>
where
	T: Config,
{
	fn is_paused(origin: &AggregateMessageOrigin) -> bool {
		// Queues for sibling parachains are paused when:
		// 1. The pallet is halted
		// 2. A higher-priority queue has pending messages
		if let Snowbridge(Sibling(_)) = origin {
			if Self::operating_mode().is_halted() {
				return true;
			}
			if PendingHighPriorityMessageCount::<T>::get() > 0 {
				return true;
			}
		}

		false
	}
}
