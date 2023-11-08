// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Types for representing inbound messages

use codec::{Decode, Encode};
use frame_support::PalletError;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// A trait for verifying inbound messages from Ethereum.
pub trait Verifier {
	fn verify(message: &Message) -> Result<(), VerificationError>;
}

#[derive(Clone, Encode, Decode, RuntimeDebug, PalletError, TypeInfo)]
#[cfg_attr(feature = "std", derive(PartialEq))]
pub enum VerificationError {
	/// Execution header is missing
	HeaderNotFound,
	/// Log was not found in the verified transaction receipt
	NotFound,
	/// Data payload does not decode into a valid Log
	InvalidLog,
	/// Unable to verify the transaction receipt with the provided proof
	InvalidProof,
}

pub type MessageNonce = u64;

/// A bridge message from the Gateway contract on Ethereum
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Message {
	/// RLP-encoded event log
	pub data: Vec<u8>,
	/// Inclusion proof for a transaction receipt containing the event log
	pub proof: Proof,
}

/// Inclusion proof for a transaction receipt
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Proof {
	// The block hash of the block in which the receipt was included.
	pub block_hash: H256,
	// The index of the transaction (and receipt) within the block.
	pub tx_index: u32,
	// Proof keys and values (receipts tree)
	pub data: (Vec<Vec<u8>>, Vec<Vec<u8>>),
}
