use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use sp_core::H160;
use sp_runtime::RuntimeDebug;

use crate::envelope::Envelope;

/// Persistent storage for inbound channels
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct InboundChannelData {
	pub nonce: u64,
}

/// Persistent storage for outbound channels
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct OutboundChannelData {
	pub nonce: u64,
}

/// Handles messages inbound from Ethereum
pub trait InboundChannel<AccountId> {
	/// Submit a message envelope for processing
	fn submit(&self, relayer: &AccountId, envelope: &Envelope) -> DispatchResult;
}

/// Handles messages outbound to Ethereum
pub trait OutboundChannel {
	/// Submit a message payload for processing
	fn submit(&self, target: H160, payload: &[u8]) -> DispatchResult;
}
