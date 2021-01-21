use frame_support::{
	dispatch::DispatchResult,
	storage::StorageValue,
};
use sp_std::{boxed::Box, cell::RefCell};
use sp_std::marker::PhantomData;


use artemis_core::{ChannelId, Message};

use crate::{
	Config,
	primitives::{InboundChannel, InboundChannelData}
};

/// Construct an inbound channel object
pub fn make_inbound_channel<T: Config>(channel_id: ChannelId) -> Box<dyn InboundChannel<T>> {
	match channel_id {
		ChannelId::Basic =>  Box::new(BasicInboundChannel::new()),
		ChannelId::Incentivized =>  Box::new(IncentivizedInboundChannel::new()),
	}
}

// Storage layer for inbound channel
struct Storage<T: Config> {
	cached_data: RefCell<Option<InboundChannelData>>,
	phantom: PhantomData<T>
}

impl<T: Config> Storage<T> {
	fn new() -> Self {
		Storage { cached_data: RefCell::new(None), phantom: PhantomData }
	}
}

impl<T: Config> Storage<T> {
	fn data(&self) -> InboundChannelData {
		match self.cached_data.clone().into_inner() {
			Some(data) => data,
			None => {
				let data = crate::InboundChannelData::get();
				*self.cached_data.try_borrow_mut().expect(
					"we're in the single-threaded environment;\
						we have no recursive borrows; qed",
				) = Some(data.clone());
				data
			}
		}
	}

	fn set_data(&mut self, data: InboundChannelData) {
		*self.cached_data.try_borrow_mut().expect(
			"we're in the single-threaded environment;\
				we have no recursive borrows; qed",
		) = Some(data.clone());
		crate::InboundChannelData::set(data)
	}

	fn try_mutate<R, E, F>(&mut self, f: F) -> Result<R, E> where
		F: FnOnce(&mut InboundChannelData) -> Result<R, E>
	{
		let mut data = self.data();
		let result = f(&mut data);
		if result.is_ok() {
			self.set_data(data)
		}
		result
	}
}

/// Basic Channel
struct BasicInboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> BasicInboundChannel<T> {
	fn new() -> Self {
		Self { storage: Storage::new() }
	}
}

impl<T: Config> InboundChannel<T> for BasicInboundChannel<T> {
	fn submit(&mut self, message: &Message) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Verifier
		//   T::ApplicationRegistry
		//   T::Rewards
		Ok(())
	}
}

/// Incentivized Channel
struct IncentivizedInboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> IncentivizedInboundChannel<T> {
	fn new() -> Self {
		Self { storage: Storage::new() }
	}
}

impl<T: Config> InboundChannel<T> for IncentivizedInboundChannel<T> {
	fn submit(&mut self, message: &Message) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Verifier
		//   T::ApplicationRegistry
		//   T::Rewards
		self.storage.try_mutate(|data|
			// Do something with data
			Ok(())
		)
	}
}
