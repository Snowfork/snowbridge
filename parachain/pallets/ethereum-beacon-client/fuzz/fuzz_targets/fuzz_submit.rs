#![no_main]
extern crate snowbridge_ethereum_beacon_client;

use std::convert::TryInto;
use snowbridge_ethereum_beacon_client::fuzzing::minimal::*;
use snowbridge_beacon_primitives::types::BeaconHeader;
use snowbridge_ethereum_beacon_client::types::{Update, NextSyncCommitteeUpdate, SyncAggregate, SyncCommittee};
use snowbridge_beacon_primitives::PublicKey;
use sp_core::H256;

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzUpdate {
	pub attested_header: FuzzBeaconHeader,
	pub sync_aggregate: FuzzSyncAggregate,
	pub signature_slot: u64,
	pub next_sync_committee_update: Option<FuzzNextSyncCommitteeUpdate>,
	pub finalized_header: FuzzBeaconHeader,
	pub finality_branch: Vec<[u8; 32]>,
	pub block_roots_root: [u8; 32],
	pub block_roots_branch: Vec<[u8; 32]>,
}

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzBeaconHeader {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: [u8; 32],
	pub state_root: [u8; 32],
	pub body_root: [u8; 32],
}

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzSyncAggregate {
	pub sync_committee_bits: [u8; 4],
	pub sync_committee_signature: [u8; 96],
}

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzNextSyncCommitteeUpdate {
	pub next_sync_committee: FuzzSyncCommittee,
	pub next_sync_committee_branch: Vec<[u8; 32]>,
}

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzSyncCommittee {
	pub pubkeys: [[u8; 48]; 32],
	pub aggregate_pubkey: [u8; 48],
}

impl TryFrom<FuzzUpdate> for Update
{
	type Error = String;

	fn try_from(other: FuzzUpdate) -> Result<Self, Self::Error> {
		let next: Option<NextSyncCommitteeUpdate> = other.next_sync_committee_update.map(|fuzz_update| fuzz_update.try_into().unwrap());

		Ok(Self {
			attested_header: other.attested_header.clone().try_into().unwrap(),
			sync_aggregate: other.sync_aggregate.try_into().unwrap(),
			signature_slot: other.signature_slot,
			next_sync_committee_update: next,
			finalized_header: other.finalized_header.clone().try_into().unwrap(),
			finality_branch: other.finality_branch.iter().map(|&hash| {
				H256::from(hash)
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
			block_roots_root: other.block_roots_root.into(),
			block_roots_branch: other.block_roots_branch.iter().map(|&hash| {
				H256::from(hash)
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
		})
	}
}

impl TryFrom<FuzzNextSyncCommitteeUpdate> for NextSyncCommitteeUpdate
{
	type Error = String;

	fn try_from(other: FuzzNextSyncCommitteeUpdate) -> Result<Self, Self::Error> {
		Ok(Self {
			next_sync_committee: SyncCommittee{
				pubkeys: other.next_sync_committee.pubkeys.iter().map(|&pk| {
					let p: PublicKey = pk.into();
					p
				}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
				aggregate_pubkey: other.next_sync_committee.aggregate_pubkey.into(),
			},
			next_sync_committee_branch: other.next_sync_committee_branch.iter().map(|&hash| {
				H256::from(hash)
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
		})
	}
}

impl TryFrom<FuzzSyncAggregate> for SyncAggregate
{
	type Error = String;

	fn try_from(other: FuzzSyncAggregate) -> Result<Self, Self::Error> {
		Ok(Self {
			sync_committee_bits: other.sync_committee_bits.into(),
			sync_committee_signature: other.sync_committee_signature.into(),
		})
	}
}

impl TryFrom<FuzzBeaconHeader> for BeaconHeader
{
	type Error = String;

	fn try_from(other: FuzzBeaconHeader) -> Result<Self, Self::Error> {
		Ok(Self {
			slot: other.slot,
			proposer_index: other.proposer_index,
			parent_root: other.parent_root.into(),
			state_root: other.state_root.into(),
			body_root: other.body_root.into(),
		})
	}
}

fuzz_target!(|input: FuzzUpdate| {
   new_tester().execute_with(|| {
		let update: Update = input.try_into().unwrap();
        let result = EthereumBeaconClient::process_update(&update);
		assert!(result.is_err());
	});
});

