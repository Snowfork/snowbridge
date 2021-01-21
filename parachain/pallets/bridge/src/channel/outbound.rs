use frame_support::{
	dispatch::DispatchResult,
	storage::StorageValue
};
use sp_std::cell::RefCell;
use sp_std::marker::PhantomData;

use crate::{
	Config,
	primitives::{OutboundChannel, OutboundChannelData}
};

use artemis_core::ChannelId;

/// Construct an inbound channel object
pub fn make_outbound_channel<T: Config>(channel_id: ChannelId) -> Box<dyn OutboundChannel<T>> {
	match channel_id {
		ChannelId::Basic =>  Box::new(BasicOutboundChannel::new()),
		ChannelId::Incentivized => Box::new(IncentivizedOutboundChannel::new()),
	}
}

// Storage layer for a channel
struct Storage<T: Config> {
	cached_data: RefCell<Option<OutboundChannelData>>,
	phantom: PhantomData<T>
}

impl<T: Config> Storage<T> {
	fn new() -> Self {
		Storage { cached_data: RefCell::new(None), phantom: PhantomData }
	}
}

impl<T: Config> Storage<T> {
	fn data(&self) -> OutboundChannelData {
		match self.cached_data.clone().into_inner() {
			Some(data) => data,
			None => {
				let data = crate::OutboundChannelData::get();
				*self.cached_data.try_borrow_mut().expect(
					"we're in the single-threaded environment;\
						we have no recursive borrows; qed",
				) = Some(data.clone());
				data
			}
		}
	}

	fn set_data(&mut self, data: OutboundChannelData) {
		*self.cached_data.try_borrow_mut().expect(
			"we're in the single-threaded environment;\
				we have no recursive borrows; qed",
		) = Some(data.clone());
		crate::OutboundChannelData::set(data)
	}

	fn try_mutate<R, E, F>(&mut self, f: F) -> Result<R, E> where
		F: FnOnce(&mut OutboundChannelData) -> Result<R, E>
	{
		let mut data = self.data();
		let result = f(&mut data);
		if result.is_ok() {
			self.set_data(data)
		}
		result
	}
}

struct BasicOutboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> BasicOutboundChannel<T> {
	fn new() -> Self {
		Self { storage: Storage::new() }
	}
}

impl<T: Config> OutboundChannel<T> for BasicOutboundChannel<T> {
	fn submit(&mut self, message: &[u8]) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Commitments
		//   T::Fees
		//   Event
		Ok(())
	}
}

struct IncentivizedOutboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> IncentivizedOutboundChannel<T> {
	fn new() -> Self {
		Self { storage: Storage::new() }
	}
}

impl<T: Config> OutboundChannel<T> for IncentivizedOutboundChannel<T> {
	fn submit(&mut self, message: &[u8]) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Commitments
		//   T::Fees
		self.storage.try_mutate(|data|
			// Do something with data
			Ok(())
		)
	}
}
