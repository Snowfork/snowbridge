use core::array::TryFromSliceError;

use crate::{config, ssz::*};
use byte_slice_cast::AsByteSlice;
use frame_support::{traits::Get, BoundedVec};
use snowbridge_beacon_primitives::{
	BeaconHeader,
	ExecutionPayload, ForkData,  SigningData,
	SyncCommittee,
};
use sp_std::{convert::TryInto, iter::FromIterator, prelude::*};
use ssz_rs::{
	prelude::{List, Vector},
	Bitvector, Deserialize, DeserializeError, SimpleSerialize as SimpleSerializeTrait,
	U256,
};

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

impl<FeeRecipientSize: Get<u32>, LogsBloomSize: Get<u32>, ExtraDataSize: Get<u32>>
TryFrom<ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>> for SSZExecutionPayload
{
	type Error = MerkleizationError;

	fn try_from(
		execution_payload: ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>,
	) -> Result<Self, Self::Error> {
		Ok(SSZExecutionPayload {
			parent_hash: execution_payload.parent_hash.as_bytes().try_into()?,
			fee_recipient: Vector::<u8, 20>::from_iter(execution_payload.fee_recipient),
			state_root: execution_payload.state_root.as_bytes().try_into()?,
			receipts_root: execution_payload.receipts_root.as_bytes().try_into()?,
			logs_bloom: Vector::<u8, 256>::from_iter(execution_payload.logs_bloom),
			prev_randao: execution_payload.prev_randao.as_bytes().try_into()?,
			block_number: execution_payload.block_number,
			gas_limit: execution_payload.gas_limit,
			gas_used: execution_payload.gas_used,
			timestamp: execution_payload.timestamp,
			extra_data: List::<u8, { config::MAX_EXTRA_DATA_BYTES }>::try_from(
				execution_payload.extra_data.into_inner(),
			)
				.map_err(|_| MerkleizationError::ListError)?,
			base_fee_per_gas: U256::try_from_bytes_le(
				&(execution_payload.base_fee_per_gas.as_byte_slice()),
			)?,
			block_hash: execution_payload.block_hash.as_bytes().try_into()?,
			transactions_root: execution_payload.transactions_root.as_bytes().try_into()?,
		})
	}
}

impl TryFrom<BeaconHeader> for SSZBeaconBlockHeader {
	type Error = MerkleizationError;

	fn try_from(beacon_header: BeaconHeader) -> Result<Self, Self::Error> {
		Ok(SSZBeaconBlockHeader {
			slot: beacon_header.slot,
			proposer_index: beacon_header.proposer_index,
			parent_root: beacon_header.parent_root.as_bytes().try_into()?,
			state_root: beacon_header.state_root.as_bytes().try_into()?,
			body_root: beacon_header.body_root.as_bytes().try_into()?,
		})
	}
}

pub fn hash_tree_root_beacon_header(
	beacon_header: BeaconHeader,
) -> Result<[u8; 32], MerkleizationError> {
	let ssz_beacon_header: SSZBeaconBlockHeader = beacon_header.try_into()?;

	hash_tree_root(ssz_beacon_header)
}

pub fn hash_tree_root_execution_header<
	FeeRecipientSize: Get<u32>,
	LogsBloomSize: Get<u32>,
	ExtraDataSize: Get<u32>,
>(
	execution_header: ExecutionPayload<
		FeeRecipientSize,
		LogsBloomSize,
		ExtraDataSize,
	>,
) -> Result<[u8; 32], MerkleizationError> {
	let ssz_execution_payload: SSZExecutionPayload = execution_header.try_into()?;

	hash_tree_root(ssz_execution_payload)
}

pub fn hash_tree_root_sync_committee<S: Get<u32>>(
	sync_committee: SyncCommittee<S>,
) -> Result<[u8; 32], MerkleizationError> {
	let mut pubkeys_vec = Vec::new();

	for pubkey in sync_committee.pubkeys.iter() {
		let conv_pubkey = Vector::<u8, 48>::from_iter(pubkey.0);

		pubkeys_vec.push(conv_pubkey);
	}

	let pubkeys =
		Vector::<Vector<u8, 48>, { config::SYNC_COMMITTEE_SIZE }>::from_iter(pubkeys_vec.clone());

	let agg = Vector::<u8, 48>::from_iter(sync_committee.aggregate_pubkey.0);

	hash_tree_root(SSZSyncCommittee { pubkeys, aggregate_pubkey: agg })
}

pub fn hash_tree_root_fork_data(fork_data: ForkData) -> Result<[u8; 32], MerkleizationError> {
	hash_tree_root(SSZForkData {
		current_version: fork_data.current_version,
		genesis_validators_root: fork_data.genesis_validators_root,
	})
}

pub fn hash_tree_root_signing_data(
	signing_data: SigningData,
) -> Result<[u8; 32], MerkleizationError> {
	hash_tree_root(SSZSigningData {
		object_root: signing_data.object_root.into(),
		domain: signing_data.domain.into(),
	})
}

pub fn hash_tree_root<T: SimpleSerializeTrait>(
	mut object: T,
) -> Result<[u8; 32], MerkleizationError> {
	match object.hash_tree_root() {
		Ok(node) => node
			.as_bytes()
			.try_into()
			.map_err(|_| MerkleizationError::HashTreeRootInvalidBytes),
		Err(_e) => Err(MerkleizationError::HashTreeRootError),
	}
}

pub fn get_sync_committee_bits<SyncCommitteeBitsSize: Get<u32>>(
	bits_hex: BoundedVec<u8, SyncCommitteeBitsSize>,
) -> Result<Vec<u8>, MerkleizationError> {
	let bitv = Bitvector::<{ config::SYNC_COMMITTEE_SIZE }>::deserialize(&bits_hex).map_err(
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

	let result = bitv.iter().map(|bit| if bit == true { 1 } else { 0 }).collect::<Vec<_>>();

	Ok(result)
}
