//! Types for representing messages

use frame_support::RuntimeDebug;
use sp_std::vec::Vec;
use sp_core::{H160, H256};
use enum_iterator::IntoEnumIterator;
use codec::{Encode, Decode};
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, IntoEnumIterator, RuntimeDebug)]
pub enum ChannelId {
	Basic,
	Incentivized
}

/// A message relayed from Ethereum.
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug)]
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
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Proof {
	// The block hash of the block in which the receipt was included.
	block_hash: H256,
	// The index of the transaction (and receipt) within the block.
	tx_index: u32,
	// Merkle proof keys and values
	merkle_proof: (Vec<Vec<u8>>, Vec<Vec<u8>>),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SourceChannelConfig {
	pub basic: SourceChannel,
	pub incentivized: SourceChannel,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SourceChannel {
	pub address: H160
}