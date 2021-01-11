//! Types for representing messages

use frame_support::RuntimeDebug;
use sp_std::vec::Vec;
use sp_core::H256;

use codec::{Encode, Decode};

/// Identifier for an application module registered within the runtime.
///
/// Typically an identifier of this type will hold an Ethereum contract address. This provides a mechanism
/// for cross-chain routing of messages.
pub type AppId = [u8; 20];

/// A message relayed from Ethereum.
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Message {
	/// The raw message payload.
	///
	/// Its content is undefined and can only be decoded by target applications.
	pub payload: Vec<u8>,

	/// Input to the message verifier
	pub verification: VerificationInput,
}

#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug)]
pub struct Messages(pub AppId, pub Vec<Message>);

/// Verification input for the message verifier.
///
/// This data type allows us to support multiple verification schemes. In the near future,
/// A light-client scheme will be added too.
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug)]
pub enum VerificationInput {
	/// Basic scheme supports replay protection
	Basic {
		/// The block number of the block in which the event was included.
		block_number: u64,
		/// The index of the event within the block.
		event_index: u32,
	},
	/// Light-client-based scheme checks receipt inclusion proof
	ReceiptProof {
		// The block hash of the block in which the receipt was included.
		block_hash: H256,
		// The index of the transaction (and receipt) within the block.
		tx_index: u32,
		// Merkle proof keys and values
		proof: (Vec<Vec<u8>>, Vec<Vec<u8>>),
	},
	/// No verification scheme. Such messages will be dropped!
	None
}
