// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
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
	pub block_roots_root: H256,
	pub block_roots_branch: Vec<H256>,
}

impl<const COMMITTEE_SIZE: usize> Default for CheckpointUpdate<COMMITTEE_SIZE> {
	fn default() -> Self {
		CheckpointUpdate {
			header: Default::default(),
			current_sync_committee: Default::default(),
			current_sync_committee_branch: Default::default(),
			validators_root: Default::default(),
			block_roots_root: Default::default(),
			block_roots_branch: Default::default(),
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
pub struct Update<const COMMITTEE_SIZE: usize, const COMMITTEE_BITS_SIZE: usize> {
	pub attested_header: BeaconHeader,
	pub sync_aggregate: SyncAggregate<COMMITTEE_SIZE, COMMITTEE_BITS_SIZE>,
	pub signature_slot: u64,
	pub next_sync_committee_update: Option<NextSyncCommitteeUpdate<COMMITTEE_SIZE>>,
	pub finalized_header: BeaconHeader,
	pub finality_branch: Vec<H256>,
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
pub struct NextSyncCommitteeUpdate<const COMMITTEE_SIZE: usize> {
	pub next_sync_committee: SyncCommittee<COMMITTEE_SIZE>,
	pub next_sync_committee_branch: Vec<H256>,
}

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(
	feature = "std",
	derive(serde::Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
pub struct ExecutionHeaderUpdate {
	/// Header for the beacon block containing the execution payload
	pub header: BeaconHeader,
	/// Proof that `header` is an ancestor of a finalized header
	pub ancestry_proof: Option<AncestryProof>,
	/// Execution header to be imported
	pub execution_header: ExecutionPayloadHeader,
	/// Merkle proof that execution payload is contained within `header`
	pub execution_branch: Vec<H256>,
}

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(
	feature = "std",
	derive(serde::Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
pub struct AncestryProof {
	/// Merkle proof that `header` is an ancestor of `finalized_header`
	pub header_branch: Vec<H256>,
	/// Root of a finalized block that has already been imported into the light client
	pub finalized_block_root: H256,
}
