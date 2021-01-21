
use crate::{
	Event,
	Config,
	RuntimeInboundChannelStorage,
	primitives
};

use artemis_core::{AppId, Application, Message, Verifier};

struct BasicOutboundChannel<T: Config> {
	data: RuntimeInboundChannelStorage<T>
}

impl<T: Config> BasicInboundChannel<T> {
	fn submit(message: &Message) -> DispatchResult {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Commitments
		//   T::Fees
		//   Event
	}
}

struct IncentivizedOutboundChannel<T: Config> {
	data: RuntimeInboundChannelStorage<T>
}

impl<T: Config> IncentivizedInboundChannel<T> {
	fn submit(message: &Message) {
		// These things are available in this scope:
		//   self.data()  // persistent data for channel
		//   T::Commitments
		//   T::Fees
		//   Event
	}
}
