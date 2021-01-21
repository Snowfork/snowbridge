use sp_runtime::RuntimeDebug;
use codec::{Encode, Decode};

type MessageNonce = u64;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ChannelId {
	Basic,
	Incentivized
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
struct InboundChannelData {
	pub nonce: u64
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
struct OutboundChannelData {
	pub nonce: u64
}
