use crate::{
	primitives::{OutboundChannel, OutboundChannelData},
	Config, Event, Module, OutboundChannels,
};
use artemis_core::{ChannelId, MessageCommitment};
use frame_support::{dispatch::DispatchResult, storage::StorageMap};
use sp_core::H160;
use sp_std::{cell::Cell, marker::PhantomData};

pub struct BasicOutboundChannel<T: Config> {
	id: ChannelId,
	storage: Storage<T>,
}

impl<T: Config> BasicOutboundChannel<T> {
	pub fn new() -> Self {
		Self {
			id: ChannelId::Basic,
			storage: Storage::new(ChannelId::Basic),
		}
	}
}

impl<T: Config> OutboundChannel for BasicOutboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&self, target: H160, payload: &[u8]) -> DispatchResult {
		self.storage.try_mutate(|data| {
			data.nonce += 1;
			T::MessageCommitment::add(self.id, target, data.nonce, payload)?;
			<Module<T>>::deposit_event(Event::MessageAccepted(self.id, data.nonce));
			Ok(())
		})
	}
}

struct Storage<T: Config> {
	channel_id: ChannelId,
	cached_data: Cell<Option<OutboundChannelData>>,
	phantom: PhantomData<T>,
}

impl<T: Config> Storage<T> {
	fn new(channel_id: ChannelId) -> Self {
		Storage {
			channel_id,
			cached_data: Cell::new(None),
			phantom: PhantomData,
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

	fn set(&self, data: OutboundChannelData) {
		self.cached_data.set(Some(data));
		OutboundChannels::insert(self.channel_id, data)
	}

	fn try_mutate<R, E, F>(&self, f: F) -> Result<R, E>
	where
		F: FnOnce(&mut OutboundChannelData) -> Result<R, E>,
	{
		let mut data = self.get();
		let result = f(&mut data);
		if result.is_ok() {
			self.set(data);
		}
		result
	}
}
