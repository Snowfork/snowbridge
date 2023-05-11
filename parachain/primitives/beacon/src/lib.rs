#![cfg_attr(not(feature = "std"), no_std)]

pub mod bits;
pub mod bls;
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
	SigningData, SyncAggregate, SyncCommittee, SyncCommitteePrepared,
};
pub use updates::{
	CheckPointUpdate, ExecutionHeaderUpdate, FinalizedHeaderUpdate, SyncCommitteeUpdate,
};

pub use bits::decompress_sync_committee_bits;
pub use bls::{
	fast_aggregate_verify, fast_aggregate_verify_legacy, prepare_aggregate_pubkey,
	prepare_aggregate_pubkey_from_absent, prepare_aggregate_signature, prepare_g1_pubkeys,
	AggregatePublicKey, AggregateSignature, BlsError, PublicKeyPrepared, SignaturePrepared,
};
pub use merkle_proof::verify_merkle_proof;
pub use receipt::verify_receipt_proof;
