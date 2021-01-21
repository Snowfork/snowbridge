use frame_support::dispatch::DispatchResult;
use sp_std::cell::RefCell;

use crate::{
	Event,
	Config,
	OutboundChannelData,
	primitives::{self, ChannelId, OutboundChannel, OutboundChannelData as ChannelData}
};

use artemis_core::{AppId, Application, Message, Verifier};

/// Construct an inbound channel object
pub fn make_outbound_channel<T: Config>(channel_id: ChannelId) -> OutboundChannel<T> {
	match channel_id {
		ChannelId::Basic => {
			BasicOutboundChannel {
				data: Storage::new()
			}
		},
		ChannelId::Incentivized => {
			IncentivizedOutboundChannel {
				data: Storage::new()
			}
		}
	}

}

// Storage layer for a channel
struct Storage<T: Config> {
	cached_data: RefCell<Option<ChannelData>>,
}

impl<T: Config> Storage<T> {
	fn new() -> Self {
		Storage { cached_data: RefCell::new(None) }
	}
}

impl<T: Config> Storage {
	fn data(&self) -> ChannelData {
		match self.cached_data.clone().into_inner() {
			Some(data) => data,
			None => {
				let data = OutboundChannelData::<T>::get(&self.lane_id);
				*self.cached_data.try_borrow_mut().expect(
					"we're in the single-threaded environment;\
						we have no recursive borrows; qed",
				) = Some(data.clone());
				data
			}
		}
	}

	fn set_data(&mut self, data: ChannelData) {
		*self.cached_data.try_borrow_mut().expect(
			"we're in the single-threaded environment;\
				we have no recursive borrows; qed",
		) = Some(data.clone());
		OutboundChannelData::<T>::set(data)
	}
}

struct BasicOutboundChannel<T: Config> {
	data: Storage<T>
}

impl<T: Config> OutboundChannel<T> for BasicOutboundChannel<T> {
	fn submit(message: &Message) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Commitments
		//   T::Fees
		//   Event
	}
}

struct IncentivizedOutboundChannel<T: Config> {
	data: Storage<T>
}

impl<T: Config> OutboundChannel<T> for IncentivizedOutboundChannel<T> {
	fn submit(message: &Message) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Commitments
		//   T::Fees
		//   Event
	}
}
