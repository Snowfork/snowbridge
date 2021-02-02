use frame_support::dispatch::DispatchResult;
use sp_runtime::RuntimeDebug;
use sp_core::H160;
use codec::{Encode, Decode};

use crate::envelope::Envelope;

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
	fn submit(&self, relayer: &AccountId, envelope: &Envelope) -> DispatchResult;
}

pub trait OutboundChannel {
	fn submit(&self, target: H160, payload: &[u8]) -> DispatchResult;
}
