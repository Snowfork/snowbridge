//! # XCMP Support
//!
//! Includes types and traits which enabling withdrawals and deposits to assets via XCMP message
//! execution.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160};

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

/// Transfers an asset to the destination parachain. Transfers failures are emitted by events.
pub trait XcmReserveTransfer<AccountId, Origin> {
	fn reserve_transfer(
		asset_id: u128,
		sender: H160,
		recipient: &AccountId,
		amount: u128,
		destination: RemoteParachain,
	);
}
