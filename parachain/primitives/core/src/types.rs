//! Types for representing messages

use codec::{Decode, Encode};
use enum_iterator::IntoEnumIterator;
use frame_support::{scale_info::TypeInfo, RuntimeDebug};
use sp_core::H256;
use sp_runtime::DigestItem;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct MessageId {
	pub channel_id: ChannelId,
	pub nonce: u64,
}

impl MessageId {
	pub fn new(channel_id: ChannelId, nonce: u64) -> Self {
		Self { channel_id, nonce }
	}
}

pub type MessageNonce = u64;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, IntoEnumIterator, RuntimeDebug, TypeInfo)]
pub enum ChannelId {
	Basic,
	Incentivized,
}

/// A message relayed from Ethereum.
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Message {
	/// The raw message data.
	pub data: Vec<u8>,
	/// Input to the message verifier
	pub proof: Proof,
}

/// Verification input for the message verifier.
///
/// This data type allows us to support multiple verification schemes. In the near future,
/// A light-client scheme will be added too.
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Proof {
	// The block hash of the block in which the receipt was included.
	pub block_hash: H256,
	// The index of the transaction (and receipt) within the block.
	pub tx_index: u32,
	// Proof keys and values
	pub data: (Vec<Vec<u8>>, Vec<Vec<u8>>),
}

/// Auxiliary [`DigestItem`] to include in header digest.
#[derive(Encode, Decode, Copy, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AuxiliaryDigestItem {
	/// A batch of messages has been committed.
	Commitment(ChannelId, H256),
}

impl<T> Into<DigestItem<T>> for AuxiliaryDigestItem {
	fn into(self) -> DigestItem<T> {
		DigestItem::Other(self.encode())
	}
}
