use frame_support::{dispatch::DispatchResult, storage::StorageMap, traits::Get};
use sp_core::H160;
use sp_std::{cell::Cell, marker::PhantomData, boxed::Box};
use artemis_core::{ChannelId, Message};
use crate::{
	Error,
	Config,
	InboundChannels,
	primitives::{InboundChannel, InboundChannelData}
};

/// Construct an inbound channel object
pub fn make_inbound_channel<T>(channel_id: ChannelId) -> Box<dyn InboundChannel<T::AccountId>>
where
	T: Config
{
	match channel_id {
		ChannelId::Basic => Box::new(BasicInboundChannel::<T>::new()),
		ChannelId::Incentivized => Box::new(IncentivizedInboundChannel::<T>::new()),
	}
}

const DUMMY_APP_ID: H160 = H160::zero();

/// Basic Channel
struct BasicInboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> BasicInboundChannel<T> {
	fn new() -> Self {
		Self {
			storage: Storage::new(ChannelId::Basic)
		}
	}
}

impl<T: Config> InboundChannel<T::AccountId> for BasicInboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&mut self, relayer: &T::AccountId, message: &Message) -> DispatchResult {
		self.storage.try_mutate(|data| {
			// Example: Increment nonce
			data.nonce += 1;

			// Example: find an app and dispatch payload
			let registry = T::Apps::get();
			registry.get(&DUMMY_APP_ID)
					.ok_or(Error::<T>::AppNotFound)?
					.handle(&message.payload)?;

			Ok(())
		})
	}
}

/// Incentivized Channel
struct IncentivizedInboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> IncentivizedInboundChannel<T> {
	fn new() -> Self {
		Self {
			storage: Storage::new(ChannelId::Incentivized)
		}
	}
}

impl<T: Config> InboundChannel<T::AccountId> for IncentivizedInboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&mut self, relayer: &T::AccountId, message: &Message) -> DispatchResult {
		Ok(())
	}
}

struct Storage<T: Config> {
	channel_id: ChannelId,
	cached_data: Cell<Option<InboundChannelData>>,
	phantom: PhantomData<T>
}

impl<T: Config> Storage<T> {
	fn new(channel_id: ChannelId) -> Self {
		Storage {
			channel_id,
			cached_data: Cell::new(None),
			phantom: PhantomData
		}
	}

	fn get(&self) -> InboundChannelData {
		match self.cached_data.get() {
			Some(data) => data,
			None => {
				let data = InboundChannels::get(self.channel_id);
				self.cached_data.set(Some(data));
				data
			}
		}
	}

	fn set(&mut self, data: InboundChannelData) {
		self.cached_data.set(Some(data));
		InboundChannels::insert(self.channel_id, data)
	}

	fn try_mutate<R, E, F>(&mut self, f: F) -> Result<R, E>
	where
		F: FnOnce(&mut InboundChannelData) -> Result<R, E>
	{
		let mut data = self.get();
		let result = f(&mut data);
		if result.is_ok() {
			self.set(data);
		}
		result
	}
}
