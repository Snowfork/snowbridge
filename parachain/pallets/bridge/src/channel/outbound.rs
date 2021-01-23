use frame_support::{
	dispatch::DispatchResult,
	storage::StorageMap,
};
use sp_core::H160;
use sp_std::{cell::Cell, marker::PhantomData, boxed::Box};
use artemis_core::{ChannelId, MessageCommitment};
use crate::{
	Config,
	OutboundChannels,
	primitives::{OutboundChannel, OutboundChannelData}
};

/// Construct an Outbound channel object
pub fn make_outbound_channel<T: Config>(channel_id: ChannelId) -> Box<dyn OutboundChannel> {
	match channel_id {
		ChannelId::Basic => Box::new(BasicOutboundChannel::<T>::new()),
		ChannelId::Incentivized => Box::new(IncentivizedOutboundChannel::<T>::new()),
	}
}

struct BasicOutboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> BasicOutboundChannel<T> {

	const CHANNEL_ID: ChannelId = ChannelId::Basic;

	fn new() -> Self {
		Self {
			storage: Storage::new(Self::CHANNEL_ID)
		}
	}
}

impl<T: Config> OutboundChannel for BasicOutboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&mut self, payload: &[u8]) -> DispatchResult {
		self.storage.try_mutate(|data| {
			T::MessageCommitment::add(Self::CHANNEL_ID, H160::zero(), data.nonce, payload);
			data.nonce += 1;
			Ok(())
		})
	}
}

struct IncentivizedOutboundChannel<T: Config> {
	storage: Storage<T>
}

impl<T: Config> IncentivizedOutboundChannel<T> {

	const CHANNEL_ID: ChannelId = ChannelId::Incentivized;

	fn new() -> Self {
		Self {
			storage: Storage::new(Self::CHANNEL_ID)
		}
	}
}

impl<T: Config> OutboundChannel for IncentivizedOutboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&mut self, payload: &[u8]) -> DispatchResult {
		self.storage.try_mutate(|data| {
			T::MessageCommitment::add(Self::CHANNEL_ID, H160::zero(), data.nonce, payload);
			data.nonce += 1;
			Ok(())
		})
	}
}

struct Storage<T: Config> {
	channel_id: ChannelId,
	cached_data: Cell<Option<OutboundChannelData>>,
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

	fn get(&self) -> OutboundChannelData {
		match self.cached_data.get() {
			Some(data) => data,
			None => {
				let data = OutboundChannels::get(self.channel_id);
				self.cached_data.set(Some(data));
				data
			}
		}
	}

	fn set(&mut self, data: OutboundChannelData) {
		self.cached_data.set(Some(data));
		OutboundChannels::insert(self.channel_id, data)
	}

	fn try_mutate<R, E, F>(&mut self, f: F) -> Result<R, E>
	where
		F: FnOnce(&mut OutboundChannelData) -> Result<R, E>
	{
		let mut data = self.get();
		let result = f(&mut data);
		if result.is_ok() {
			self.set(data);
		}
		result
	}
}
