use frame_support::dispatch::DispatchResult;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Represents a remote parachain by id with a fee that will be used by
/// `XcmReserveTransfer::reserve_transfer` to send an asset to a remote
/// parachain.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct RemoteParachain {
	/// The parachain id.
	pub para_id: u32,
	/// The fee required for XCM execution.
	pub fee: u128,
}

pub trait XcmReserveTransfer<AccountId, Origin> {
	fn reserve_transfer(
		asset_id: u128,
		recipient: &AccountId,
		amount: u128,
		destination: RemoteParachain,
	) -> DispatchResult;
}
