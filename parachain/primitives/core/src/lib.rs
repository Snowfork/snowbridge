//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use artemis_ethereum::Log;
use frame_support::dispatch::{DispatchError, DispatchResult};
use sp_core::H160;

pub mod assets;
pub mod types;

pub use types::{ChannelId, Message, MessageId, Proof, SourceChannel, SourceChannelConfig};

pub use assets::{AssetId, MultiAsset, SingleAsset};
/// A trait for verifying messages.
///
/// This trait should be implemented by runtime modules that wish to provide message verification functionality.
pub trait Verifier {
	fn verify(message: &Message) -> Result<Log, DispatchError>;
}

/// Outbound submission for applications
pub trait SubmitOutbound<AccountId> {
	fn submit(account_id: AccountId, target: H160, payload: &[u8]) -> DispatchResult;
}

/// Outbound submission for applications, specifying ChannelId
pub trait SubmitOutboundChannel {
	fn submit(channel_id: ChannelId, target: H160, payload: &[u8]) -> DispatchResult;
}

pub trait BasicMessageCommitment<AccountId> {
	fn add_basic(account_id: AccountId, target: H160, nonce: u64, payload: &[u8])
		-> DispatchResult;
}

/// Add a message to a commitment
pub trait IncentivizedMessageCommitment {
	fn add_incentivized(target: H160, nonce: u64, payload: &[u8]) -> DispatchResult;
}

/// Dispatch a message
pub trait MessageDispatch<MessageId> {
	fn dispatch(source: H160, id: MessageId, payload: &[u8]);
}
