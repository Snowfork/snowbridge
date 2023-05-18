use codec::{Decode, Encode};
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_std::prelude::*;

use crate::types::{BeaconHeader, ExecutionPayloadHeader, SyncAggregate, SyncCommittee};

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(
	feature = "std",
	derive(serde::Serialize, serde::Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
pub struct CheckpointUpdate<const COMMITTEE_SIZE: usize> {
	pub header: BeaconHeader,
	pub current_sync_committee: SyncCommittee<COMMITTEE_SIZE>,
	pub current_sync_committee_branch: Vec<H256>,
	pub validators_root: H256,
	pub import_time: u64,
}

impl<const COMMITTEE_SIZE: usize> Default for CheckpointUpdate<COMMITTEE_SIZE> {
	fn default() -> Self {
		CheckpointUpdate {
			header: Default::default(),
			current_sync_committee: Default::default(),
			current_sync_committee_branch: Default::default(),
			validators_root: Default::default(),
			import_time: Default::default(),
		}
	}
}

#[derive(
	Default, Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[cfg_attr(
	feature = "std",
	derive(serde::Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
pub struct SyncCommitteeUpdate<const COMMITTEE_SIZE: usize, const COMMITTEE_BITS_SIZE: usize> {
	pub attested_header: BeaconHeader,
	pub next_sync_committee: SyncCommittee<COMMITTEE_SIZE>,
	pub next_sync_committee_branch: Vec<H256>,
	pub finalized_header: BeaconHeader,
	pub finality_branch: Vec<H256>,
	pub sync_aggregate: SyncAggregate<COMMITTEE_SIZE, COMMITTEE_BITS_SIZE>,
	pub signature_slot: u64,
	pub block_roots_root: H256,
	pub block_roots_branch: Vec<H256>,
}

#[derive(
	Default, Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[cfg_attr(
	feature = "std",
	derive(serde::Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
pub struct FinalizedHeaderUpdate<const COMMITTEE_SIZE: usize, const COMMITTEE_BITS_SIZE: usize> {
	pub attested_header: BeaconHeader,
	pub finalized_header: BeaconHeader,
	pub finality_branch: Vec<H256>,
	pub sync_aggregate: SyncAggregate<COMMITTEE_SIZE, COMMITTEE_BITS_SIZE>,
	pub signature_slot: u64,
	pub block_roots_root: H256,
	pub block_roots_branch: Vec<H256>,
}

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(
	feature = "std",
	derive(serde::Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
pub struct ExecutionHeaderUpdate {
	pub header: BeaconHeader,
	pub execution_header: ExecutionPayloadHeader,
	pub execution_branch: Vec<H256>,
	pub block_roots_root: H256,
	pub block_roots_branch: Vec<H256>,
}
