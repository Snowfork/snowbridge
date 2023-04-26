use core::array::TryFromSliceError;

use crate::{
	config, ssz::*, BeaconHeader, ExecutionPayloadHeader, ForkData, SigningData, SyncAggregate,
};

use sp_std::{convert::TryInto, iter::FromIterator, prelude::*};
use ssz_rs::{
	prelude::{List, Vector},
	Bitvector, Deserialize, DeserializeError, SimpleSerialize as SimpleSerializeTrait, U256,
};

use sp_core::H256;

use super::SyncCommittee;

use config::{EXTRA_DATA_SIZE, LOGS_BLOOM_SIZE};

#[derive(Debug, PartialEq)]
pub enum MerkleizationError {
	HashTreeRootError,
	HashTreeRootInvalidBytes,
	InvalidLength,
	ExpectedFurtherInput { provided: u64, expected: u64 },
	AdditionalInput { provided: u64, expected: u64 },
	InvalidInput,
	DeserializeError,
	ListError,
}

impl From<TryFromSliceError> for MerkleizationError {
	fn from(_: TryFromSliceError) -> Self {
		return MerkleizationError::InvalidLength
	}
}

impl From<DeserializeError> for MerkleizationError {
	fn from(_: DeserializeError) -> Self {
		return MerkleizationError::DeserializeError
	}
}

pub fn hash_tree_root_beacon_header(
	beacon_header: BeaconHeader,
) -> Result<H256, MerkleizationError> {
	hash_tree_root::<SSZBeaconBlockHeader>(beacon_header.into())
}

pub fn hash_tree_root_execution_header(
	execution_payload_header: ExecutionPayloadHeader,
) -> Result<H256, MerkleizationError> {
	hash_tree_root::<SSZExecutionPayloadHeader>(execution_payload_header.into())
}

pub fn hash_tree_root_sync_committee<const SYNC_COMMITTEE_SIZE: usize>(
	sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
) -> Result<H256, MerkleizationError> {
	hash_tree_root::<SSZSyncCommittee<SYNC_COMMITTEE_SIZE>>(sync_committee.into())
}

pub fn hash_tree_root_fork_data(fork_data: ForkData) -> Result<H256, MerkleizationError> {
	hash_tree_root::<SSZForkData>(fork_data.into())
}

pub fn hash_tree_root_signing_data(signing_data: SigningData) -> Result<H256, MerkleizationError> {
	hash_tree_root::<SSZSigningData>(signing_data.into())
}

pub fn hash_tree_root<T: SimpleSerializeTrait>(mut object: T) -> Result<H256, MerkleizationError> {
	match object.hash_tree_root() {
		Ok(node) => {
			let fixed_bytes: [u8; 32] = node
				.as_bytes()
				.try_into()
				.map_err(|_| MerkleizationError::HashTreeRootInvalidBytes)?;
			Ok(fixed_bytes.into())
		},
		Err(_e) => Err(MerkleizationError::HashTreeRootError),
	}
}

pub fn get_sync_committee_bits<const SYNC_COMMITTEE_SIZE: usize>(
	input: &[u8],
) -> Result<[u8; SYNC_COMMITTEE_SIZE], MerkleizationError> {
	let bitv = Bitvector::<{ SYNC_COMMITTEE_SIZE }>::deserialize(input).map_err(
		//|_| MerkleizationError::InvalidInput
		|e| -> MerkleizationError {
			match e {
				DeserializeError::ExpectedFurtherInput { provided, expected } =>
					MerkleizationError::ExpectedFurtherInput {
						provided: provided as u64,
						expected: expected as u64,
					},
				DeserializeError::AdditionalInput { provided, expected } =>
					MerkleizationError::AdditionalInput {
						provided: provided as u64,
						expected: expected as u64,
					},
				_ => MerkleizationError::InvalidInput,
			}
		},
	)?;

	let cleaned = bitv.iter().map(|bit| if bit == true { 1u8 } else { 0u8 }).collect::<Vec<u8>>();

	cleaned.try_into().map_err(|_| MerkleizationError::InvalidInput)
}
