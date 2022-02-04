#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchError;

/// Allocate the next available asset id
pub trait NextAssetId {
	fn next() -> Result<u128, DispatchError>;
}
