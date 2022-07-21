use ssz_rs_derive::SimpleSerialize;
use ssz_rs::{Deserialize, Sized, Bitlist, Bitvector, U256};
use ssz_rs::prelude::{Vector, List};
use sp_std::{vec::Vec, vec};

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZVoluntaryExit {
    pub epoch: u64,
    pub validator_index: u64,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZDepositData {
    pub pubkey: Vector<u8, 48>,
    pub withdrawal_credentials: [u8; 32],
    pub amount: u64,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZDeposit {
    pub proof: Vector<[u8; 32], { crate::DEPOSIT_CONTRACT_TREE_DEPTH + 1 }>,
    pub data: SSZDepositData,
}

#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZCheckpoint {
    pub epoch: u64,
    pub root: [u8; 32],
}

#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZAttestationData {
    pub slot: u64,
    pub index: u64,
    pub beacon_block_root: [u8; 32],
    pub source: SSZCheckpoint,
    pub target: SSZCheckpoint,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SignedBeaconBlockHeader {
    pub message: SSZBeaconBlockHeader,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZIndexedAttestation {
    pub attesting_indices: List<u64, { crate::MAX_VALIDATORS_PER_COMMITTEE }>,
    pub data: SSZAttestationData,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZAttesterSlashing {
    pub attestation_1: SSZIndexedAttestation,
    pub attestation_2: SSZIndexedAttestation,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZEth1Data {
    pub deposit_root: [u8; 32],
    pub deposit_count: u64,
    pub block_hash: [u8; 32],
}

#[derive(Default, SimpleSerialize, Clone, Debug)]
pub struct SSZAttestation {
    pub aggregation_bits: Bitlist<{ crate::MAX_VALIDATORS_PER_COMMITTEE} >,
    pub data: SSZAttestationData,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, SimpleSerialize)]
pub struct SSZBeaconBlock<const SYNC_COMMITTEE_SIZE: usize> {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: [u8; 32],
    pub state_root: [u8; 32],
    pub body: SSZBeaconBlockBody<SYNC_COMMITTEE_SIZE>,
}

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
    pub pubkeys: Vector<Vector<u8, 48>, 512>,
    pub aggregate_pubkey: Vector<u8, 48>,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZSyncAggregate<const SYNC_COMMITTEE_SIZE: usize> {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Vector<u8, 96>,
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
    pub fee_recipient: Vector<u8, 20>,
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub logs_bloom: Vector<u8, 256>,
    pub prev_randao: [u8; 32],
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: List<u8, { crate::MAX_EXTRA_DATA_BYTES }>,
    pub base_fee_per_gas: U256,
    pub block_hash: [u8; 32],
    pub transactions_root: [u8; 32],
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
pub struct SSZBeaconBlockBody<const SYNC_COMMITTEE_SIZE: usize> {
    pub randao_reveal: Vector<u8, 96>,
    pub eth1_data: SSZEth1Data,
    pub graffiti: [u8; 32],
    pub proposer_slashings: List<SSZProposerSlashing, { crate::MAX_PROPOSER_SLASHINGS }>,
    pub attester_slashings: List<SSZAttesterSlashing, { crate::MAX_ATTESTER_SLASHINGS }>,
    pub attestations: List<SSZAttestation, { crate::MAX_ATTESTATIONS }>,
    pub deposits: List<SSZDeposit, { crate::MAX_DEPOSITS }>,
    pub voluntary_exits: List<SSZVoluntaryExit, { crate::MAX_VOLUNTARY_EXITS }>,
    pub sync_aggregate: SSZSyncAggregate<SYNC_COMMITTEE_SIZE>,
    pub execution_payload: SSZExecutionPayload,
}
