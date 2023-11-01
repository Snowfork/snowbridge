//! Implementation for [`frame_support::traits::QueuePausedQuery`]
use super::*;
use frame_support::traits::QueuePausedQuery;
use snowbridge_core::outbound::{AggregateMessageOrigin, ExportOrigin};

impl<T> QueuePausedQuery<AggregateMessageOrigin> for Pallet<T>
where
	T: Config,
{
	fn is_paused(origin: &AggregateMessageOrigin) -> bool {
		use AggregateMessageOrigin::*;
		use ExportOrigin::*;

		// Queues for sibling parachains are paused when:
		// 1. The pallet is halted
		// 2. A higher-priority queue has pending messages
		if let Export(Sibling(_)) = origin {
			if Self::operating_mode().is_halted() {
				return true
			}
			if PendingHighPriorityMessageCount::<T>::get() > 0 {
				return true
			}
		}

		false
	}
}
