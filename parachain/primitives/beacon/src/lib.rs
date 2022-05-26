#![cfg_attr(not(feature = "std"), no_std)]

use scale_info::TypeInfo;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use sp_core::{H160, H256, U256};

pub type Root = H256;
pub type Domain = H256;
pub type ValidatorIndex = u64;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey(pub [u8; 48]);

impl Default for PublicKey {
	fn default() -> Self {
		PublicKey([0u8; 48])
	}
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ForkData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SigningData {
	pub object_root: Root,
	pub domain: Domain,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Header {
	pub parent_hash: H256,
	pub fee_recipient: H160,
	pub state_root: H256,
	pub receipts_root: H256,
	pub logs_bloom: Vec<u8>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_used: U256,
	pub gas_limit: U256,
	pub timestamp: u64,
	pub extra_data: Vec<u8>,
	pub base_fee_per_gas: Option<U256>,
	pub block_hash: H256,
	pub transactions: Vec<u8>,
}

/// Sync committee as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncCommittee {
	pub pubkeys: Vec<PublicKey>,
	pub aggregate_pubkey: PublicKey,
}

/// Beacon block header as it is stored in the runtime storage. The block root is the
/// Merklization of a BeaconHeader.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BeaconHeader {
	// The slot for which this block is created. Must be greater than the slot of the block defined by parentRoot.
	pub slot: u64,
	// The index of the validator that proposed the block.
	pub proposer_index: ValidatorIndex,
	// The block root of the parent block, forming a block chain.
	pub parent_root: Root,
	// The hash root of the post state of running the state transition through this block.
	pub state_root: Root,
	// The hash root of the beacon block body
	pub body_root: Root,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct DepositData {
	pub pubkey: Vec<u8>,
	pub withdrawal_credentials: H256,
	pub amount: u64,
	pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Deposit {
	pub proof: Vec<H256>,
	pub data: DepositData,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Checkpoint {
	pub epoch: u64,
	pub root: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AttestationData {
	pub slot: u64,
	pub index: u64,
	pub beacon_block_root: H256,
	pub source: Checkpoint,
	pub target: Checkpoint,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct IndexedAttestation {
    pub attesting_indices: Vec<u64>,
    pub data: AttestationData,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignedHeader {
	pub message: crate::BeaconHeader,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ProposerSlashing {
	pub signed_header_1: SignedHeader,
	pub signed_header_2: SignedHeader,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AttesterSlashing {
	pub attestation_1: IndexedAttestation,
	pub attestation_2: IndexedAttestation,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Attestation {
	pub aggregation_bits: Vec<u8>,
	pub data: AttestationData,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct VoluntaryExit {
	pub epoch: u64,
	pub validator_index: u64,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Eth1Data {
	pub deposit_root: H256,
	pub deposit_count: u64,
	pub block_hash: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncAggregate {
	pub sync_committee_bits: Vec<u8>,
	pub sync_committee_signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExecutionPayload {
	pub parent_hash: H256,
	pub fee_recipient: Vec<u8>,
	pub state_root: H256,
	pub receipts_root: H256,
	pub logs_bloom: Vec<u8>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: Vec<u8>,
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions: Vec<Vec<u8>>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Body {
	pub randao_reveal: Vec<u8>,
	pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: Vec<ProposerSlashing>,
    pub attester_slashings: Vec<AttesterSlashing>,
    pub attestations: Vec<Attestation>,
    pub deposits: Vec<Deposit>,
    pub voluntary_exits: Vec<VoluntaryExit>,
    pub sync_aggregate: SyncAggregate,
    pub execution_payload: ExecutionPayload,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BeaconBlock {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: H256,
	pub state_root: H256,
	pub body: Body,
}
