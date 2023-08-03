#![no_main]
extern crate snowbridge_ethereum_beacon_client;

use std::convert::TryInto;
use snowbridge_ethereum_beacon_client::mock::minimal::*;
use snowbridge_beacon_primitives::updates::AncestryProof;
use snowbridge_beacon_primitives::{ExecutionPayloadHeader, ExecutionHeaderUpdate, BeaconHeader};
use sp_core::H256;

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzExecutionHeaderUpdate {
	pub header: FuzzBeaconHeader,
	pub ancestry_proof: Option<FuzzAncestryProof>,
	pub execution_header: FuzzExecutionPayloadHeader,
	pub execution_branch: Vec<[u8; 32]>,
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
pub struct FuzzAncestryProof {
	pub header_branch: Vec<[u8; 32]>,
	pub finalized_block_root: [u8; 32],
}

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzExecutionPayloadHeader {
	pub parent_hash: [u8; 32],
	pub fee_recipient: [u8; 20],
	pub state_root: [u8; 32],
	pub receipts_root: [u8; 32],
	pub logs_bloom: Vec<u8>,
	pub prev_randao: [u8; 32],
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: Vec<u8>,
	pub base_fee_per_gas: u128,
	pub block_hash: [u8; 32],
	pub transactions_root: [u8; 32],
	pub withdrawals_root: [u8; 32],
}

impl TryFrom<FuzzAncestryProof> for AncestryProof
{
	type Error = String;

	fn try_from(other: FuzzAncestryProof) -> Result<Self, Self::Error> {
		Ok(Self {
			header_branch: other.header_branch.iter().map(|&hash| {
				H256::from(hash)
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
			finalized_block_root: other.finalized_block_root.into(),
		})
	}
}

impl TryFrom<FuzzExecutionPayloadHeader> for ExecutionPayloadHeader
{
	type Error = String;

	fn try_from(other: FuzzExecutionPayloadHeader) -> Result<Self, Self::Error> {
		Ok(Self {
			parent_hash: other.parent_hash.into(),
			fee_recipient: other.fee_recipient.into(),
			state_root: other.state_root.into(),
			receipts_root: other.receipts_root.into(),
			logs_bloom: other.logs_bloom.into(),
			prev_randao: other.prev_randao.into(),
			block_number: other.block_number,
			gas_limit: other.gas_limit,
			gas_used: other.gas_used,
			timestamp: other.timestamp,
			extra_data: other.extra_data.into(),
			base_fee_per_gas: other.base_fee_per_gas.into(),
			block_hash: other.block_hash.into(),
			transactions_root: other.transactions_root.into(),
			withdrawals_root: other.withdrawals_root.into(),
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

impl TryFrom<FuzzExecutionHeaderUpdate> for ExecutionHeaderUpdate
{
	type Error = String;

	fn try_from(other: FuzzExecutionHeaderUpdate) -> Result<Self, Self::Error> {
		let ancestry_proof: Option<AncestryProof> = other.ancestry_proof.map(|fuzz_update| fuzz_update.try_into().unwrap());

		Ok(Self {
			header: other.header.try_into().unwrap(),
			ancestry_proof: ancestry_proof,
			execution_header: other.execution_header.try_into().unwrap(),
			execution_branch: other.execution_branch.iter().map(|&hash| {
				H256::from(hash)
			}).collect::<Vec<_>>().as_slice().try_into().unwrap(),
		})
	}
}

fuzz_target!(|input: FuzzExecutionHeaderUpdate| {
   new_tester().execute_with(|| {
		let update: ExecutionHeaderUpdate = input.try_into().unwrap();
        let result = EthereumBeaconClient::process_execution_header_update(&update);
		assert!(result.is_err());
	});
});

