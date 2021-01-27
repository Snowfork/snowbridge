use frame_support::dispatch::DispatchResult;
use sp_runtime::RuntimeDebug;
use codec::{Encode, Decode};
use artemis_core::{AppId, Message};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct InboundChannelData {
	pub nonce: u64
}
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct OutboundChannelData {
	pub nonce: u64
}

pub trait InboundChannel<AccountId>
{
	fn submit(&mut self, relayer: &AccountId, app_id: AppId, message: &Message) -> DispatchResult;
}

pub trait OutboundChannel {
	fn submit(&self, payload: &[u8]) -> DispatchResult;
}
