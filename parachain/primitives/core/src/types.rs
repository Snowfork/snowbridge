//! Types for representing messages

use sp_std::vec::Vec;
use sp_core::H160;

use codec::{Encode, Decode};

/// Identifier for an application module registered within the runtime.
///
/// Typically an identifier of this type will hold an Ethereum contract address. This provides a mechanism
/// for cross-chain routing of messages.
pub type AppId = [u8; 20];

/// A message relayed from Ethereum.
#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct Message {
	/// The raw message payload.
	///
	/// Its content is undefined and can only be decoded by target applications.
	pub payload: Vec<u8>,

	/// Input to the message verifier
	pub verification: VerificationInput,
}

/// Verification input for the message verifier.
///
/// This data type allows us to support multiple verification schemes. In the near future,
/// A light-client scheme will be added too.
#[derive(Debug, PartialEq, Copy, Clone, Encode, Decode)]
pub enum VerificationInput {
	/// Basic scheme supports replay protection
	Basic {
		/// The block number of the block in which the event was included.
		block_number: u64,
		/// The index of the event within the block.
		event_index: u32,
	},
	/// No verification scheme. Such messages will be dropped!
	None
}

/// ID for Bridged Assets
pub type BridgedAssetId = H160;
