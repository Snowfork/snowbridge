#![no_main]
extern crate snowbridge_ethereum_beacon_client;

use std::convert::TryInto;
use snowbridge_ethereum_beacon_client::fuzzing::minimal::*;
use snowbridge_beacon_primitives::types::BeaconHeader;
use snowbridge_ethereum_beacon_client::types::{SyncCommittee, CheckpointUpdate};
use snowbridge_beacon_primitives::PublicKey;
use sp_core::H256;

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzCheckpointUpdate {
	pub header: FuzzBeaconHeader,
	pub current_sync_committee: FuzzSyncCommittee,
	pub current_sync_committee_branch: Vec<[u8; 32]>,
	pub validators_root: [u8; 32],
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
pub struct FuzzSyncCommittee {
	pub pubkeys: [[u8; 48]; 32],
	pub aggregate_pubkey: [u8; 48],
}

impl TryFrom<FuzzCheckpointUpdate> for CheckpointUpdate
{
	type Error = String;

	fn try_from(other: FuzzCheckpointUpdate) -> Result<Self, Self::Error> {
		Ok(Self {
			header: other.header.clone().try_into().unwrap(),
			current_sync_committee: other.current_sync_committee.try_into().unwrap(),
			current_sync_committee_branch: other.current_sync_committee_branch.iter().map(|&hash| {
				H256::from(hash)
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
			validators_root: other.validators_root.into(),
			block_roots_root: other.block_roots_root.into(),
			block_roots_branch: other.block_roots_branch.iter().map(|&hash| {
				H256::from(hash)
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
		})
	}
}

impl TryFrom<FuzzSyncCommittee> for SyncCommittee
{
	type Error = String;

	fn try_from(other: FuzzSyncCommittee) -> Result<Self, Self::Error> {
		Ok(Self{
			pubkeys: other.pubkeys.iter().map(|&pk| {
				let p: PublicKey = pk.into();
				p
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
			//pubkeys: Default::default(),
			aggregate_pubkey: other.aggregate_pubkey.into(),
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

fuzz_target!(|input: FuzzCheckpointUpdate| {
   new_tester().execute_with(|| {
		let update: CheckpointUpdate = input.try_into().unwrap();
        let _result = EthereumBeaconClient::process_checkpoint_update(&update);
		//assert!();
	});
});

