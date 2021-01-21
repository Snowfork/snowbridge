
use frame_support::dispatch::DispatchResult;
use sp_runtime::RuntimeDebug;
use codec::{Encode, Decode};

use crate::Config;

use artemis_core::Message;


#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ChannelId {
	Basic,
	Incentivized
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct InboundChannelData {
	// Used by incentivized channel, ignored by basic channel
	pub nonce: u64
}
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OutboundChannelData {
	// Used by incentivized channel, ignored by basic channel
	pub nonce: u64
}

pub trait InboundChannel<T: Config> {
	fn submit(message: &Message) -> DispatchResult;
}

pub trait OutboundChannel<T: Config> {
	fn submit(message: &[u8]) -> DispatchResult;
}
