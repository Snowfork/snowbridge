use crate::{
	config::{EXTRA_DATA_SIZE, FEE_RECIPIENT_SIZE, LOGS_BLOOM_SIZE, PUBKEY_SIZE, SIGNATURE_SIZE},
	types::{
		BeaconHeader, ExecutionPayloadHeader, ForkData, SigningData, SyncAggregate, SyncCommittee,
	},
};
use byte_slice_cast::AsByteSlice;
use sp_core::H256;
use sp_std::{vec, vec::Vec};
use ssz_rs::{
	prelude::{List, Vector},
	Bitvector, Deserialize, MerkleizationError, SimpleSerialize, Sized, U256,
};
use ssz_rs_derive::SimpleSerialize as SimpleSerializeDerive;

#[derive(Default, SimpleSerializeDerive, Clone, Debug)]
pub struct SSZBeaconBlockHeader {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: [u8; 32],
	pub state_root: [u8; 32],
	pub body_root: [u8; 32],
}

impl From<BeaconHeader> for SSZBeaconBlockHeader {
	fn from(beacon_header: BeaconHeader) -> Self {
		SSZBeaconBlockHeader {
			slot: beacon_header.slot,
			proposer_index: beacon_header.proposer_index,
			parent_root: beacon_header.parent_root.to_fixed_bytes(),
			state_root: beacon_header.state_root.to_fixed_bytes(),
			body_root: beacon_header.body_root.to_fixed_bytes(),
		}
	}
}

#[derive(Default, SimpleSerializeDerive, Clone)]
pub struct SSZSyncCommittee<const COMMITTEE_SIZE: usize> {
	pub pubkeys: Vector<Vector<u8, PUBKEY_SIZE>, COMMITTEE_SIZE>,
	pub aggregate_pubkey: Vector<u8, PUBKEY_SIZE>,
}

impl<const COMMITTEE_SIZE: usize> From<SyncCommittee<COMMITTEE_SIZE>>
	for SSZSyncCommittee<COMMITTEE_SIZE>
{
	fn from(sync_committee: SyncCommittee<COMMITTEE_SIZE>) -> Self {
		let mut pubkeys_vec = Vec::new();

		for pubkey in sync_committee.pubkeys.iter() {
			let conv_pubkey = Vector::<u8, PUBKEY_SIZE>::from_iter(pubkey.0);

			pubkeys_vec.push(conv_pubkey);
		}

		let pubkeys =
			Vector::<Vector<u8, PUBKEY_SIZE>, { COMMITTEE_SIZE }>::from_iter(pubkeys_vec.clone());

		let aggregate_pubkey =
			Vector::<u8, PUBKEY_SIZE>::from_iter(sync_committee.aggregate_pubkey.0);

		SSZSyncCommittee { pubkeys, aggregate_pubkey }
	}
}

#[derive(Default, Debug, SimpleSerializeDerive, Clone)]
pub struct SSZSyncAggregate<const COMMITTEE_SIZE: usize> {
	pub sync_committee_bits: Bitvector<COMMITTEE_SIZE>,
	pub sync_committee_signature: Vector<u8, SIGNATURE_SIZE>,
}

impl<const COMMITTEE_SIZE: usize, const COMMITTEE_BITS_SIZE: usize>
	From<SyncAggregate<COMMITTEE_SIZE, COMMITTEE_BITS_SIZE>> for SSZSyncAggregate<COMMITTEE_SIZE>
{
	fn from(sync_aggregate: SyncAggregate<COMMITTEE_SIZE, COMMITTEE_BITS_SIZE>) -> Self {
		SSZSyncAggregate {
			sync_committee_bits: Bitvector::<COMMITTEE_SIZE>::deserialize(
				&sync_aggregate.sync_committee_bits,
			)
			.expect("checked statically; qed"),
			sync_committee_signature: Vector::<u8, SIGNATURE_SIZE>::from_iter(
				sync_aggregate.sync_committee_signature.0,
			),
		}
	}
}

#[derive(Default, SimpleSerializeDerive, Clone)]
pub struct SSZForkData {
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

impl From<ForkData> for SSZForkData {
	fn from(fork_data: ForkData) -> Self {
		SSZForkData {
			current_version: fork_data.current_version,
			genesis_validators_root: fork_data.genesis_validators_root,
		}
	}
}

#[derive(Default, SimpleSerializeDerive, Clone)]
pub struct SSZSigningData {
	pub object_root: [u8; 32],
	pub domain: [u8; 32],
}

impl From<SigningData> for SSZSigningData {
	fn from(signing_data: SigningData) -> Self {
		SSZSigningData {
			object_root: signing_data.object_root.into(),
			domain: signing_data.domain.into(),
		}
	}
}

#[derive(Default, SimpleSerializeDerive, Clone, Debug)]
pub struct SSZExecutionPayloadHeader {
	pub parent_hash: [u8; 32],
	pub fee_recipient: Vector<u8, FEE_RECIPIENT_SIZE>,
	pub state_root: [u8; 32],
	pub receipts_root: [u8; 32],
	pub logs_bloom: Vector<u8, LOGS_BLOOM_SIZE>,
	pub prev_randao: [u8; 32],
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: List<u8, EXTRA_DATA_SIZE>,
	pub base_fee_per_gas: U256,
	pub block_hash: [u8; 32],
	pub transactions_root: [u8; 32],
	pub withdrawals_root: [u8; 32],
}

impl From<ExecutionPayloadHeader> for SSZExecutionPayloadHeader {
	fn from(payload: ExecutionPayloadHeader) -> Self {
		SSZExecutionPayloadHeader {
			parent_hash: payload.parent_hash.to_fixed_bytes(),
			fee_recipient: Vector::<u8, FEE_RECIPIENT_SIZE>::from_iter(
				payload.fee_recipient.to_fixed_bytes(),
			),
			state_root: payload.state_root.to_fixed_bytes(),
			receipts_root: payload.receipts_root.to_fixed_bytes(),
			logs_bloom: Vector::<u8, 256>::from_iter(payload.logs_bloom),
			prev_randao: payload.prev_randao.to_fixed_bytes(),
			block_number: payload.block_number,
			gas_limit: payload.gas_limit,
			gas_used: payload.gas_used,
			timestamp: payload.timestamp,
			extra_data: List::<u8, EXTRA_DATA_SIZE>::from_iter(payload.extra_data),
			base_fee_per_gas: U256::from_bytes_le(
				payload
					.base_fee_per_gas
					.as_byte_slice()
					.try_into()
					.expect("checked in prep; qed"),
			),
			block_hash: payload.block_hash.to_fixed_bytes(),
			transactions_root: payload.transactions_root.to_fixed_bytes(),
			withdrawals_root: payload.withdrawals_root.to_fixed_bytes(),
		}
	}
}

pub fn hash_tree_root<T: SimpleSerialize>(mut object: T) -> Result<H256, MerkleizationError> {
	match object.hash_tree_root() {
		Ok(node) => {
			let fixed_bytes: [u8; 32] =
				node.as_bytes().try_into().expect("Node is a newtype over [u8; 32]; qed");
			Ok(fixed_bytes.into())
		},
		Err(err) => Err(err),
	}
}
