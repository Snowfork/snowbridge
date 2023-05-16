use codec::{Decode, Encode};
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::Deserialize;

use crate::types::{BeaconHeader, ExecutionPayloadHeader, SyncAggregate, SyncCommittee};

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(
	feature = "std",
	derive(serde::Serialize, serde::Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize))]
#[codec(mel_bound())]
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
#[scale_info(skip_type_params(SyncCommitteeSize))]
#[codec(mel_bound())]
pub struct SyncCommitteeUpdate<const COMMITTEE_SIZE: usize, const COMMITTEE_BITS_SIZE: usize> {
	pub attested_header: BeaconHeader,
	pub next_sync_committee: SyncCommittee<COMMITTEE_SIZE>,
	pub next_sync_committee_branch: Vec<H256>,
	pub finalized_header: BeaconHeader,
	pub finality_branch: Vec<H256>,
	pub sync_aggregate: SyncAggregate<COMMITTEE_SIZE, COMMITTEE_BITS_SIZE>,
	pub sync_committee_period: u64,
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
#[scale_info(skip_type_params(SyncCommitteeSize,))]
#[codec(mel_bound())]
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
	derive(Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize))]
#[codec(mel_bound())]
pub struct ExecutionHeaderUpdate<const COMMITTEE_SIZE: usize, const COMMITTEE_BITS_SIZE: usize> {
	pub beacon_header: BeaconHeader,
	pub execution_header: ExecutionPayloadHeader,
	pub execution_branch: Vec<H256>,
	pub sync_aggregate: SyncAggregate<COMMITTEE_SIZE, COMMITTEE_BITS_SIZE>,
	pub signature_slot: u64,
	pub block_root_branch: Vec<H256>,
	pub block_root_branch_header_root: H256,
}
