//! # XCMP Support
//!
//! Includes types and traits which enabling withdrawals and deposits to assets via XCMP message
//! execution.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160, H256};

/// Represents a remote parachain by id with a fee that will be used by
/// `XcmReserveTransfer::reserve_transfer` to send an asset to a remote
/// parachain.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct RemoteParachain {
	/// The parachain id.
	pub para_id: u32,
	/// The fee required for XCM execution.
	pub fee: u128,
}

/// Represents information about an XCM transfer.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct TransferInfo {
	/// The asset id on our parachain.
	pub asset_id: u128,
	// The senders Ethereum address.
	pub sender: H160,
	// The recipient Substrate address.
	pub recipient: H256,
	// The amount transfered.
	pub amount: u128,
	/// The destination parachain id.
	pub para_id: u32,
	/// The fee paid for the xcm request.
	pub fee: u128,
}

/// Transfers an asset to the destination parachain. Transfer failures are emitted by events.
pub trait XcmReserveTransfer<AccountId, RuntimeOrigin> {
	fn reserve_transfer(
		asset_id: u128,
		sender: H160,
		recipient: &AccountId,
		amount: u128,
		destination: RemoteParachain,
	);
}
