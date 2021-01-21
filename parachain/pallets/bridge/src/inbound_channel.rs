
use crate::{
	Event,
	Config,
	RuntimeInboundChannelStorage,
	primitives
};

use artemis_core::{AppId, Application, Message, Verifier};

struct BasicInboundChannel<T: Config> {
	data: RuntimeInboundChannelStorage<T>
}

impl<T: Config> BasicInboundChannel<T> {
	fn submit(message: &Message) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Verifier
		//   T::ApplicationRegistry
		//   T::Rewards
		//   Event
	}
}

struct IncentivizedInboundChannel<T: Config> {
	data: RuntimeInboundChannelStorage<T>
}

impl<T: Config> IncentivizedInboundChannel<T> {
	fn submit(message: &Message) {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Verifier
		//   T::ApplicationRegistry
		//   T::Rewards
		//   Event
	}
}
