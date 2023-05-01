#![cfg_attr(not(feature = "std"), no_std)]

pub mod bits;
pub mod config;
pub mod merkle_proof;
pub mod receipt;
pub mod ssz;
pub mod types;
pub mod updates;

#[cfg(feature = "std")]
mod serde_utils;

pub use types::{
	BeaconHeader, CompactExecutionHeader, ExecutionHeaderState, ExecutionPayloadHeader,
	FinalizedHeaderState, Fork, ForkData, ForkVersion, ForkVersions, PublicKey, Signature,
	SigningData, SyncAggregate, SyncCommittee,
};
pub use updates::{FinalizedHeaderUpdate, HeaderUpdate, InitialUpdate, SyncCommitteeUpdate};

pub use bits::decompress_sync_committee_bits;
pub use merkle_proof::verify_merkle_proof;
pub use receipt::verify_receipt_proof;
