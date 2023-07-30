// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Types for representing inbound messages

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchError;
use frame_support::{scale_info::TypeInfo, RuntimeDebug};
use snowbridge_ethereum::Log;
use sp_core::H256;
use sp_std::vec::Vec;

/// A trait for verifying inbound messages from Ethereum.
///
/// This trait should be implemented by runtime modules that wish to provide message verification
/// functionality.
pub trait Verifier {
	fn verify(message: &Message) -> Result<Log, DispatchError>;
}

pub type MessageNonce = u64;

/// A message relayed from Ethereum.
#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Message {
	/// The raw RLP-encoded message data.
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
	// Proof keys and values (receipts tree)
	pub data: (Vec<Vec<u8>>, Vec<Vec<u8>>),
}
