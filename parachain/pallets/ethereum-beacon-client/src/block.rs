
use sp_core::H256;
use scale_info::TypeInfo;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

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
pub struct Message {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: H256,
	pub state_root: H256,
	pub body_root: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Vote {
	pub slot: u64,
	pub index: u64,
	pub beacon_block_root: H256,
	pub source: Checkpoint,
	pub target: Checkpoint,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AttestationSlashing {
    pub attesting_indices: Vec<u64>,
    pub data: Vote,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignedHeader {
	pub message: Message,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ProposerSlashing {
	pub signed_header_1: SignedHeader,
	pub signed_header_2: SignedHeader,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AttesterSlashing {
	pub attestation_1: AttestationSlashing,
	pub attestation_2: AttestationSlashing,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Attestation { 
	pub aggregation_bits: Vec<u8>,
	pub data: Vote,
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
	pub extra_data: H256,
	pub base_fee_per_gas: u64,
	pub block_hash: H256,
	pub transactions_root: H256,
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