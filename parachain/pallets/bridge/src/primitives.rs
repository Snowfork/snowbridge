use frame_support::dispatch::{Parameter, DispatchResult};
use sp_runtime::RuntimeDebug;
use codec::{Encode, Decode};
use artemis_core::Message;

use sp_std::fmt::Debug;
use sp_runtime::traits::{Member, MaybeSerializeDeserialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct InboundChannelData {
	pub nonce: u64
}
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct OutboundChannelData {
	pub nonce: u64
}

pub trait InboundChannel<AccountId>
where
	AccountId: Parameter + Member + Ord + MaybeSerializeDeserialize + Debug
{
	fn submit(&mut self, relayer: &AccountId, message: &Message) -> DispatchResult;
}

pub trait OutboundChannel {
	fn submit(&mut self, payload: &[u8]) -> DispatchResult;
}
