use crate::config;
use sp_std::{vec, vec::Vec};
use ssz_rs::{
	prelude::{List, Vector},
	Bitvector, Deserialize, Sized, U256,
};
use ssz_rs_derive::SimpleSerialize;

#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZBeaconBlockHeader {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: [u8; 32],
	pub state_root: [u8; 32],
	pub body_root: [u8; 32],
}

#[derive(Default, SimpleSerialize)]
pub struct SSZSyncCommittee {
	pub pubkeys: Vector<Vector<u8, { config::PUBKEY_SIZE }>, { config::SYNC_COMMITTEE_SIZE }>,
	pub aggregate_pubkey: Vector<u8, { config::PUBKEY_SIZE }>,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZSyncAggregate {
	pub sync_committee_bits: Bitvector<{ config::SYNC_COMMITTEE_SIZE }>,
	pub sync_committee_signature: Vector<u8, { config::SIGNATURE_SIZE }>,
}

#[derive(Default, SimpleSerialize)]
pub struct SSZForkData {
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(Default, SimpleSerialize)]
pub struct SSZSigningData {
	pub object_root: [u8; 32],
	pub domain: [u8; 32],
}

#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZExecutionPayload {
	pub parent_hash: [u8; 32],
	pub fee_recipient: Vector<u8, { config::MAX_FEE_RECIPIENT_SIZE }>,
	pub state_root: [u8; 32],
	pub receipts_root: [u8; 32],
	pub logs_bloom: Vector<u8, { config::MAX_LOGS_BLOOM_SIZE }>,
	pub prev_randao: [u8; 32],
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: List<u8, { config::MAX_EXTRA_DATA_BYTES }>,
	pub base_fee_per_gas: U256,
	pub block_hash: [u8; 32],
	pub transactions_root: [u8; 32],
}

#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZExecutionPayloadCapella {
	pub parent_hash: [u8; 32],
	pub fee_recipient: Vector<u8, { config::MAX_FEE_RECIPIENT_SIZE }>,
	pub state_root: [u8; 32],
	pub receipts_root: [u8; 32],
	pub logs_bloom: Vector<u8, { config::MAX_LOGS_BLOOM_SIZE }>,
	pub prev_randao: [u8; 32],
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: List<u8, { config::MAX_EXTRA_DATA_BYTES }>,
	pub base_fee_per_gas: U256,
	pub block_hash: [u8; 32],
	pub transactions_root: [u8; 32],
	pub withdrawals_root: [u8; 32],
}
