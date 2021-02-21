use crate::{
	primitives::{OutboundChannel, OutboundChannelData},
	Config, Event, Module, OutboundChannels,
};
use artemis_core::MessageCommitment;
use frame_support::{dispatch::DispatchResult, storage::StorageMap};
use frame_system::{self as system};
use sp_core::H160;
use sp_std::{cell::Cell, marker::PhantomData};

pub struct BasicOutboundChannel<T: Config> {
	account_id: T::AccountId,
	storage: Storage<T>,
}

impl<T: Config> BasicOutboundChannel<T> {
	pub fn new(account_id: T::AccountId) -> Self {
		Self {
			account_id: account_id.clone(),
			storage: Storage::new(account_id), // TODO: clean this up
		}
	}
}

impl<T: Config> OutboundChannel for BasicOutboundChannel<T> {
	// This implementation is a WIP!
	fn submit(&self, target: H160, payload: &[u8]) -> DispatchResult {
		self.storage.try_mutate(|data| {
			data.nonce += 1;
			T::MessageCommitment::add(self.account_id.clone(), target, data.nonce, payload)?;
			//<Module<T>>::deposit_event(Event::MessageAccepted(self.account_id, data.nonce));
			Ok(())
		})
	}
}

struct Storage<T: Config> {
	account_id: T::AccountId,
	cached_data: Cell<Option<OutboundChannelData>>,
	phantom: PhantomData<T>,
}

impl<T: Config> Storage<T> {
	fn new(account_id: T::AccountId) -> Self {
		Storage {
			account_id,
			cached_data: Cell::new(None),
			phantom: PhantomData,
		}
	}

	fn get(&self) -> OutboundChannelData {
		match self.cached_data.get() {
			Some(data) => data,
			None => {
				let data = OutboundChannels::<T>::get(self.account_id.clone()); // TODO: can we avoid the clone?
				self.cached_data.set(Some(data));
				data
			}
		}
	}

	fn set(&self, data: OutboundChannelData) {
		self.cached_data.set(Some(data));
		OutboundChannels::<T>::insert(self.account_id.clone(), data) // TODO: can we avoid the clone?
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
