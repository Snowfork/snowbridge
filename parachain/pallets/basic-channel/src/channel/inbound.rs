use crate::{
	envelope::Envelope,
	primitives::{InboundChannel, InboundChannelData},
	Config, Error, InboundChannels,
};
use artemis_core::{ChannelId, MessageDispatch, MessageId};
use artemis_ethereum::H160;
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	storage::StorageMap,
};
use sp_std::{cell::Cell, marker::PhantomData};

/// Basic Channel
pub struct BasicInboundChannel<T: Config> {
	eth_address: H160,
	storage: Storage<T>,
}

impl<T: Config> BasicInboundChannel<T> {
	pub fn new(eth_address: H160) -> Self {
		Self {
			eth_address,
			storage: Storage::new(eth_address),
		}
	}
}

impl<T: Config> InboundChannel<T::AccountId> for BasicInboundChannel<T> {
	fn submit(&self, _relayer: &T::AccountId, envelope: &Envelope) -> DispatchResult {
		self.storage.try_mutate::<_, DispatchError, _>(|data| {
			if envelope.nonce != data.nonce + 1 {
				return Err(Error::<T>::BadNonce.into());
			}
			data.nonce += 1;
			Ok(())
		})?;

		let message_id = MessageId::new(ChannelId::Basic, self.eth_address, envelope.nonce);
		T::MessageDispatch::dispatch(envelope.source, message_id, &envelope.payload);

		Ok(())
	}
}

struct Storage<T: Config> {
	eth_address: H160,
	cached_data: Cell<Option<InboundChannelData>>,
	phantom: PhantomData<T>,
}

impl<T: Config> Storage<T> {
	fn new(eth_address: H160) -> Self {
		Storage {
			eth_address,
			cached_data: Cell::new(None),
			phantom: PhantomData,
		}
	}

	#[allow(dead_code)]
	fn get(&self) -> InboundChannelData {
		match self.cached_data.get() {
			Some(data) => data,
			None => {
				let data = InboundChannels::get(self.eth_address);
				self.cached_data.set(Some(data));
				data
			}
		}
	}

	#[allow(dead_code)]
	fn set(&self, data: InboundChannelData) {
		self.cached_data.set(Some(data));
		InboundChannels::insert(self.eth_address, data)
	}

	#[allow(dead_code)]
	fn try_mutate<R, E, F>(&self, f: F) -> Result<R, E>
	where
		F: FnOnce(&mut InboundChannelData) -> Result<R, E>,
	{
		let mut data = self.get();
		let result = f(&mut data);
		if result.is_ok() {
			self.set(data);
		}
		result
	}
}
