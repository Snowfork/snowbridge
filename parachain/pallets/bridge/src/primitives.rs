
use frame_support::dispatch::DispatchResult;
use sp_runtime::RuntimeDebug;
use codec::{Encode, Decode};

use crate::Config;

use artemis_core::Message;


#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct InboundChannelData {
	// Used by incentivized channel, ignored by basic channel
	pub nonce: u64
}
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub struct OutboundChannelData {
	// Used by incentivized channel, ignored by basic channel
	pub nonce: u64
}

pub trait InboundChannel<T: Config> {
	fn submit(&mut self, message: &Message) -> DispatchResult;
}

pub trait OutboundChannel<T: Config> {
	fn submit(&mut self, message: &[u8]) -> DispatchResult;
}
