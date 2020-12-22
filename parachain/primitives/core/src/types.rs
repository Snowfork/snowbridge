//! Types for representing messages

use frame_support::RuntimeDebug;
use sp_std::vec::Vec;
use sp_core::H256;

use artemis_ethereum::Receipt;
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

/// Verification output returned by the message verifier
/// 
/// This data type allows us to return a value that has been verified. The primary use case
/// is returning the value proven by an inclusion proof, e.g. ReceiptProof returns Receipt.
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug)]
pub enum VerificationOutput {
	/// The receipt for which inclusion was proven. Corresponds to ReceiptProof input.
	Receipt(Receipt),
	/// Verification has no output aside from Ok / Err.
	None
}
