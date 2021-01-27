use frame_support::{dispatch::DispatchResult, storage::StorageMap};
use sp_std::{cell::Cell, marker::PhantomData, boxed::Box};
use artemis_core::{ChannelId, AppId, Message, Verifier};
use crate::{
	Module,
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

/// Basic Channel
struct BasicInboundChannel<T: Config> {
	#[allow(dead_code)]
	channel_id: ChannelId,
	#[allow(dead_code)]
	storage: Storage<T>
}

impl<T: Config> BasicInboundChannel<T> {
	fn new() -> Self {
		Self {
			channel_id: ChannelId::Basic,
			storage: Storage::new(ChannelId::Basic)
		}
	}
}

impl<T: Config> InboundChannel<T::AccountId> for BasicInboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&mut self, relayer: &T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		T::Verifier::verify(relayer.clone(), app_id, &message)?;
		Module::<T>::dispatch(app_id.into(), message)
	}
}

/// Incentivized Channel
struct IncentivizedInboundChannel<T: Config> {
	#[allow(dead_code)]
	channel_id: ChannelId,
	#[allow(dead_code)]
	storage: Storage<T>
}

impl<T: Config> IncentivizedInboundChannel<T> {
	fn new() -> Self {
		Self {
			channel_id: ChannelId::Incentivized,
			storage: Storage::new(ChannelId::Incentivized)
		}
	}
}

impl<T: Config> InboundChannel<T::AccountId> for IncentivizedInboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&mut self, relayer: &T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		T::Verifier::verify(relayer.clone(), app_id, &message)?;
		Module::<T>::dispatch(app_id.into(), message)
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

	#[allow(dead_code)]
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

	#[allow(dead_code)]
	fn set(&mut self, data: InboundChannelData) {
		self.cached_data.set(Some(data));
		InboundChannels::insert(self.channel_id, data)
	}

	#[allow(dead_code)]
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
