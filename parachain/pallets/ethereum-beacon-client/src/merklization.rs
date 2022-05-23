use crate::{BeaconHeader, SyncCommittee, ForkData, SigningData, block::BeaconBlock, block::ExecutionPayload, block::Attestation, block::AttestationData, block::Checkpoint};

use ssz_rs_derive::SimpleSerialize;
use ssz_rs::{Deserialize, Sized, SimpleSerialize as SimpleSerializeTrait, Bitlist};
use ssz_rs::prelude::{Vector, List};
use sp_std::convert::TryInto;
use sp_std::iter::FromIterator;
use sp_std::prelude::*;

const MAX_PROPOSER_SLASHINGS: usize = 16;

const MAX_ATTESTER_SLASHINGS: usize =  2;

const MAX_ATTESTATIONS: usize =  128;

const MAX_DEPOSITS: usize =  16;

const MAX_VOLUNTARY_EXITS: usize =  16;

const MAX_VALIDATORS_PER_COMMITTEE: usize = 2048;

const DEPOSIT_CONTRACT_TREE_DEPTH: usize = 32;

const MAX_BYTES_PER_TRANSACTION: usize = 1073741824;

const MAX_TRANSACTIONS_PER_PAYLOAD: usize = 1048576;

const MAX_EXTRA_DATA_BYTES: usize = 32;

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZVoluntaryExit {
	pub epoch: u64,
	pub validator_index: u64,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZDepositData {
	pub pubkey: Vector<u8, 48>,
	pub withdrawal_credentials: [u8; 32],
	pub amount: u64,
	pub signature: Vector<u8, 96>,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZDeposit {
	pub proof: Vector<[u8; 32], {DEPOSIT_CONTRACT_TREE_DEPTH + 1}>,
	pub data: SSZDepositData,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZCheckpoint {
	pub epoch: u64,
	pub root: [u8; 32],
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZMessage {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: [u8; 32],
	pub state_root: [u8; 32],
	pub body_root: [u8; 32],
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZAttestationData {
	pub slot: u64,
	pub index: u64,
	pub beacon_block_root: [u8; 32],
	pub source: SSZCheckpoint,
	pub target: SSZCheckpoint,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZSignedHeader {
	pub message: SSZMessage,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZAttestationSlashing {
    pub attesting_indices: Vector<u64, MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: SSZAttestationData,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZProposerSlashing {
	pub signed_header_1: SSZSignedHeader,
	pub signed_header_2: SSZSignedHeader,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZAttesterSlashing {
	pub attestation_1: SSZAttestationSlashing,
	pub attestation_2: SSZAttestationSlashing,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZEth1Data {
	pub deposit_root: [u8; 32],
	pub deposit_count: u64,
	pub block_hash: [u8; 32],
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZAttestation { 
	pub aggregation_bits: Bitlist<MAX_VALIDATORS_PER_COMMITTEE>,
	pub data: SSZAttestationData,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZAttestationTest { 
	pub aggregation_bits: [u8; 32],
	pub data: SSZAttestationData,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, SimpleSerialize)]
pub struct SSZBeaconBlock {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: [u8; 32],
	pub state_root: [u8; 32],
	pub body: SSZBeaconBlockBody,
}

#[derive(Default, SimpleSerialize)]
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

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZSyncAggregate {
	pub sync_committee_bits: Vector<u8, 64>,
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

#[derive(Default, SimpleSerialize, Clone)]
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
	pub extra_data: List<u8, MAX_EXTRA_DATA_BYTES>,
	pub base_fee_per_gas: u64,
	pub block_hash: [u8; 32],
	pub transactions: List<List<u8, MAX_BYTES_PER_TRANSACTION>, MAX_TRANSACTIONS_PER_PAYLOAD>,
}

#[derive(Debug)]
pub enum MerkleizationError {
    HashTreeRootError,
    HashTreeRootInvalidBytes,
    InvalidLength
}

#[derive(Default, SimpleSerialize, Clone)]
pub struct SSZBeaconBlockBody {
	pub randao_reveal: Vector<u8, 96>,
    pub eth1_data: SSZEth1Data,
    pub graffiti: [u8; 32],
    pub proposer_slashings: Vector<SSZProposerSlashing, MAX_PROPOSER_SLASHINGS>,
    pub attester_slashings: Vector<SSZAttesterSlashing, MAX_ATTESTER_SLASHINGS>,
    pub attestations: Vector<SSZAttestation, MAX_ATTESTATIONS>,
    pub deposit: Vector<SSZDeposit, MAX_DEPOSITS>,
    pub voluntary_exits: Vector<SSZVoluntaryExit, MAX_VOLUNTARY_EXITS>, 
    pub sync_aggregate: SSZSyncAggregate,
    pub execution_payload: SSZExecutionPayload, 
}

pub fn hash_tree_root_beacon_block(beacon_block: BeaconBlock) -> Result<[u8; 32], MerkleizationError> {
    let conv_randao_reveal = Vector::<u8, 96>::from_iter(beacon_block.body.randao_reveal);

    let mut proposer_slashings = Vec::new();

    for proposer_slashing in beacon_block.body.proposer_slashings.iter() {
        let signature1 = Vector::<u8, 96>::from_iter(proposer_slashing.signed_header_1.signature.clone());
        let signature2 = Vector::<u8, 96>::from_iter(proposer_slashing.signed_header_2.signature.clone());

        let conv_proposer_slashing = SSZProposerSlashing{
            signed_header_1: SSZSignedHeader{
                message: SSZMessage{
                    slot: proposer_slashing.signed_header_1.message.slot,
                    proposer_index: proposer_slashing.signed_header_1.message.proposer_index,
                    parent_root: proposer_slashing.signed_header_1.message.parent_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    state_root: proposer_slashing.signed_header_1.message.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    body_root: proposer_slashing.signed_header_1.message.body_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                },
                signature: signature1,
            },
            signed_header_2: SSZSignedHeader{
                message: SSZMessage{
                    slot: proposer_slashing.signed_header_2.message.slot,
                    proposer_index: proposer_slashing.signed_header_2.message.proposer_index,
                    parent_root: proposer_slashing.signed_header_2.message.parent_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    state_root: proposer_slashing.signed_header_2.message.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    body_root: proposer_slashing.signed_header_2.message.body_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                },
                signature: signature2,
            },
        };

        proposer_slashings.push(conv_proposer_slashing);
    }

    let proposer_slashings_conv = Vector::<SSZProposerSlashing, MAX_PROPOSER_SLASHINGS>::from_iter(proposer_slashings);

    let mut attester_slashings = Vec::new();

    for attester_slashing in beacon_block.body.attester_slashings.iter() {
        let signature1 = Vector::<u8, 96>::from_iter(attester_slashing.attestation_1.signature.clone());
        let signature2 = Vector::<u8, 96>::from_iter(attester_slashing.attestation_2.signature.clone());

        let attesting_indices1 = Vector::<u64, MAX_VALIDATORS_PER_COMMITTEE>::from_iter(attester_slashing.attestation_1.attesting_indices.clone());
        let attesting_indices2 = Vector::<u64, MAX_VALIDATORS_PER_COMMITTEE>::from_iter(attester_slashing.attestation_2.attesting_indices.clone());

        let conv_attestor_slashing = SSZAttesterSlashing{
            attestation_1: SSZAttestationSlashing{
                attesting_indices: attesting_indices1,
                data: SSZAttestationData{
                    slot: attester_slashing.attestation_1.data.slot,
                    index: attester_slashing.attestation_1.data.index,
                    beacon_block_root: attester_slashing.attestation_1.data.beacon_block_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    source: SSZCheckpoint{
                        epoch: attester_slashing.attestation_1.data.source.epoch,
                        root: attester_slashing.attestation_1.data.source.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    },
                    target: SSZCheckpoint{
                        epoch: attester_slashing.attestation_1.data.target.epoch,
                        root: attester_slashing.attestation_1.data.target.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    },
                },
                signature: signature1,
            },
            attestation_2: SSZAttestationSlashing{
                attesting_indices: attesting_indices2,
                data: SSZAttestationData{
                    slot: attester_slashing.attestation_2.data.slot,
                    index: attester_slashing.attestation_2.data.index,
                    beacon_block_root: attester_slashing.attestation_2.data.beacon_block_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    source: SSZCheckpoint{
                        epoch: attester_slashing.attestation_2.data.source.epoch,
                        root: attester_slashing.attestation_2.data.source.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    },
                    target: SSZCheckpoint{
                        epoch: attester_slashing.attestation_2.data.target.epoch,
                        root: attester_slashing.attestation_2.data.target.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                    },
                },
                signature: signature2,
            },
        };

        attester_slashings.push(conv_attestor_slashing);
    }

    let attester_slashings_conv = Vector::<SSZAttesterSlashing, MAX_ATTESTER_SLASHINGS>::from_iter(attester_slashings);

    let mut attestations = Vec::new();

    for attestation in beacon_block.body.attestations.iter() {
        let signature = Vector::<u8, 96>::from_iter(attestation.signature.clone());

        let agg_bits = convert_to_binary_bool(attestation.aggregation_bits.clone());

        let conv_attestor_attestation = SSZAttestation{
            aggregation_bits: Bitlist::<MAX_VALIDATORS_PER_COMMITTEE>::from_iter(agg_bits),
            data: SSZAttestationData{
                slot: attestation.data.slot,
                index: attestation.data.index,
                beacon_block_root: attestation.data.beacon_block_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                source: SSZCheckpoint{
                    epoch: attestation.data.source.epoch,
                    root: attestation.data.source.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                },
                target: SSZCheckpoint{
                    epoch: attestation.data.target.epoch,
                    root: attestation.data.target.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                },
            },
            signature: signature,
        };

        attestations.push(conv_attestor_attestation);
    }

    let attestations_conv = Vector::<SSZAttestation, MAX_ATTESTATIONS>::from_iter(attestations);


    let mut voluntary_exits = Vec::new();

    for voluntary_exit in beacon_block.body.voluntary_exits.iter() {
        voluntary_exits.push(SSZVoluntaryExit{
            epoch: voluntary_exit.epoch,
            validator_index: voluntary_exit.validator_index,
        });
    }

    let voluntary_exits_conv = Vector::<SSZVoluntaryExit, MAX_VOLUNTARY_EXITS>::from_iter(voluntary_exits);

    let mut deposits = Vec::new();

    for deposit in beacon_block.body.deposits.iter() {
        let mut proofs = Vec::new();

        for proof in deposit.proof.iter() {
            proofs.push(proof.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,)
        }

        let proofs_conv = Vector::<[u8; 32], {DEPOSIT_CONTRACT_TREE_DEPTH + 1}>::from_iter(proofs);

        deposits.push(SSZDeposit{
            proof: proofs_conv,
            data: SSZDepositData{
                pubkey: Vector::<u8, 48>::from_iter(deposit.data.pubkey.clone()),
                withdrawal_credentials: deposit.data.withdrawal_credentials.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                amount: deposit.data.amount,
                signature: Vector::<u8, 96>::from_iter(deposit.data.signature.clone()),
            }
        });
    }

    let deposit_data_conv = Vector::<SSZDeposit, MAX_DEPOSITS>::from_iter(deposits);

    let mut transactions = Vec::new();

    for transaction in beacon_block.body.execution_payload.transactions.iter() {
        transactions.push(List::<u8, MAX_BYTES_PER_TRANSACTION>::from_iter((*transaction).clone()));
    }

    let transactions_conv = List::<List::<u8, MAX_BYTES_PER_TRANSACTION>, MAX_TRANSACTIONS_PER_PAYLOAD>::try_from(transactions).map_err(|_| MerkleizationError::InvalidLength)?;

    let ssz_block = SSZBeaconBlock{
        slot: beacon_block.slot,
        proposer_index: beacon_block.proposer_index,
        parent_root: beacon_block.parent_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        state_root: beacon_block.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        body: SSZBeaconBlockBody{
            randao_reveal: conv_randao_reveal,
            eth1_data: SSZEth1Data{
                deposit_root: beacon_block.body.eth1_data.deposit_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
	            deposit_count: beacon_block.body.eth1_data.deposit_count,
                block_hash: beacon_block.body.eth1_data.block_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
            },
            graffiti: beacon_block.body.graffiti.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
            proposer_slashings: proposer_slashings_conv,
            attester_slashings: attester_slashings_conv,
            attestations: attestations_conv,
            deposit: deposit_data_conv,
            voluntary_exits: voluntary_exits_conv,
            sync_aggregate: SSZSyncAggregate{
                sync_committee_bits: Vector::<u8, 64>::from_iter(beacon_block.body.sync_aggregate.sync_committee_bits),
                sync_committee_signature: Vector::<u8, 96>::from_iter(beacon_block.body.sync_aggregate.sync_committee_signature),
            },
            execution_payload: SSZExecutionPayload{
                parent_hash: beacon_block.body.execution_payload.parent_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                fee_recipient: Vector::<u8, 20>::from_iter(beacon_block.body.execution_payload.fee_recipient),
                state_root: beacon_block.body.execution_payload.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                receipts_root: beacon_block.body.execution_payload.receipts_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                logs_bloom: Vector::<u8, 256>::from_iter(beacon_block.body.execution_payload.logs_bloom),
                prev_randao: beacon_block.body.execution_payload.prev_randao.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                block_number: beacon_block.body.execution_payload.block_number,
                gas_limit: beacon_block.body.execution_payload.gas_limit,
                gas_used: beacon_block.body.execution_payload.gas_used,
                timestamp: beacon_block.body.execution_payload.timestamp,
                extra_data: List::<u8, MAX_EXTRA_DATA_BYTES>::try_from(beacon_block.body.execution_payload.extra_data.to_vec()).map_err(|_| MerkleizationError::InvalidLength)?,
                base_fee_per_gas: beacon_block.body.execution_payload.base_fee_per_gas,
                block_hash: beacon_block.body.execution_payload.block_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                transactions: transactions_conv,
            }
        },
    };

    hash_tree_root(ssz_block)
}

pub fn hash_tree_root_execution_payload(execution_payload: ExecutionPayload) -> Result<[u8; 32], MerkleizationError> {
    let mut transactions = Vec::new();

    for transaction in execution_payload.transactions.iter() {
        transactions.push(List::<u8, MAX_BYTES_PER_TRANSACTION>::from_iter((*transaction).clone()));
    }

    let transactions_conv = List::<List::<u8, MAX_BYTES_PER_TRANSACTION>, MAX_TRANSACTIONS_PER_PAYLOAD>::try_from(transactions).map_err(|_| MerkleizationError::InvalidLength)?;

    let ssz_execution_payload = SSZExecutionPayload{
        parent_hash: execution_payload.parent_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        fee_recipient: Vector::<u8, 20>::from_iter(execution_payload.fee_recipient),
        state_root: execution_payload.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        receipts_root: execution_payload.receipts_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        logs_bloom: Vector::<u8, 256>::from_iter(execution_payload.logs_bloom),
        prev_randao: execution_payload.prev_randao.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        block_number: execution_payload.block_number,
        gas_limit: execution_payload.gas_limit,
        gas_used: execution_payload.gas_used,
        timestamp: execution_payload.timestamp,
        extra_data: List::<u8, MAX_EXTRA_DATA_BYTES>::try_from(execution_payload.extra_data).map_err(|_| MerkleizationError::InvalidLength)?,
        base_fee_per_gas: execution_payload.base_fee_per_gas,
        block_hash: execution_payload.block_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        transactions: transactions_conv,
    };

    hash_tree_root(ssz_execution_payload)
}

pub fn hash_tree_root_attestation(attestation: Attestation) -> Result<[u8; 32], MerkleizationError> {
        let signature = Vector::<u8, 96>::from_iter(attestation.signature.clone());

        let agg_bits = convert_to_binary_bool(attestation.aggregation_bits);

        let conv_attestor_attestation = SSZAttestation{
            aggregation_bits: Bitlist::<MAX_VALIDATORS_PER_COMMITTEE>::from_iter(agg_bits),
            data: SSZAttestationData{
                slot: attestation.data.slot,
                index: attestation.data.index,
                beacon_block_root: attestation.data.beacon_block_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                source: SSZCheckpoint{
                    epoch: attestation.data.source.epoch,
                    root: attestation.data.source.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                },
                target: SSZCheckpoint{
                    epoch: attestation.data.target.epoch,
                    root: attestation.data.target.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                },
            },
            signature: signature,
        };

    hash_tree_root(conv_attestor_attestation)
}

pub fn hash_tree_root_attestation_data(attestation_data: AttestationData) -> Result<[u8; 32], MerkleizationError> {
    let conv_attestation_data = SSZAttestationData{
            slot: attestation_data.slot,
            index: attestation_data.index,
            beacon_block_root: attestation_data.beacon_block_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
            source: SSZCheckpoint{
                epoch: attestation_data.source.epoch,
                root: attestation_data.source.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
            },
            target: SSZCheckpoint{
                epoch: attestation_data.target.epoch,
                root: attestation_data.target.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
            },
        };

    hash_tree_root(conv_attestation_data)
}

pub fn hash_tree_root_checkpoint(checkpoint: Checkpoint) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(SSZCheckpoint{
        epoch: checkpoint.epoch,
        root: checkpoint.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    })
}

pub fn hash_tree_root_beacon_header(beacon_header: BeaconHeader) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(SSZBeaconBlockHeader{
        slot: beacon_header.slot,
        proposer_index: beacon_header.proposer_index,
        parent_root: beacon_header.parent_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        state_root: beacon_header.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        body_root: beacon_header.body_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    })
}

pub fn hash_tree_root_sync_committee(sync_committee: SyncCommittee) -> Result<[u8; 32], MerkleizationError> {
    let mut pubkeys_vec = Vec::new();

    for pubkey in sync_committee.pubkeys.iter() {
        let conv_pubkey = Vector::<u8, 48>::from_iter(pubkey.0);

        pubkeys_vec.push(conv_pubkey);
    }

    let pubkeys = Vector::<Vector::<u8, 48>, 512>::from_iter(pubkeys_vec.clone());

    let agg = Vector::<u8, 48>::from_iter(sync_committee.aggregate_pubkey.0);

    hash_tree_root(SSZSyncCommittee{ 
        pubkeys: pubkeys, 
        aggregate_pubkey: agg,
    })
}

pub fn hash_tree_root_fork_data(fork_data: ForkData) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(SSZForkData{ 
        current_version: fork_data.current_version, 
        genesis_validators_root: fork_data.genesis_validators_root
    })
}

pub fn hash_tree_root_signing_data(signing_data: SigningData) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(SSZSigningData{ 
        object_root: signing_data.object_root.into(),
        domain: signing_data.domain.into(),
    })
}

pub fn hash_tree_root<T: SimpleSerializeTrait>(mut object: T) -> Result<[u8; 32], MerkleizationError> {
    match object.hash_tree_root() {
        Ok(node)=> node.as_bytes().try_into().map_err(|_| MerkleizationError::HashTreeRootInvalidBytes), 
        Err(_e) => Err(MerkleizationError::HashTreeRootError)
    }
}

pub fn convert_to_binary_bool(input: Vec<u8>) -> Vec<bool> {
    let mut result = Vec::new();

    for input_decimal in input.iter() {
        let mut tmp = Vec::new();
        let mut remaining = *input_decimal;

        while remaining > 0 {
            let remainder = remaining % 2;
            if remainder == 1 {
                tmp.push(true);
            } else {
                tmp.push(false);
            }
            
            remaining = remaining / 2;
        }

        // pad binary with 0s if length is less than 7
        if tmp.len() < 8 {
            for _i in tmp.len()..8 {
                tmp.push(false);
            }
        }

        result.append(&mut tmp);
    }

    result
}

pub fn convert_to_binary_bool_2(input: Vec<u8>) -> Vec<bool> {
    let mut result = Vec::new();

    for input_decimal in input.iter() {
        let mut tmp = Vec::new();
        let mut remaining = *input_decimal;

        while remaining > 0 {
            let remainder = remaining % 2;
            if remainder == 1 {
                tmp.push(true);
            } else {
                tmp.push(false);
            }
            
            remaining = remaining / 2;
        }

        // pad binary with 0s if length is less than 7
        if tmp.len() < 8 {
            for _i in tmp.len()..8 {
                tmp.push(false);
            }
        }

        result.append(&mut tmp);
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::block::{AttestationData, Checkpoint, AttestationSlashing, AttesterSlashing, Body, BeaconBlock, Eth1Data, Attestation, ExecutionPayload, SyncAggregate};
    use crate::merklization::{self, SSZCheckpoint};
    use crate as ethereum_beacon_client;
    use frame_support::{assert_ok};
    use sp_core::H256;
    use ssz_rs::Bitvector;
    use ssz_rs::prelude::Vector;
    use crate::merklization::SSZAttestationTest;
    use crate::merklization::SSZAttestationData;
    use ssz_rs::Deserialize;
    use ssz_rs::{List, Bitlist};

    use hex_literal::hex;


    #[test]
    pub fn test_hash_tree_root_beacon_header() {
        let hash_root = merklization::hash_tree_root_beacon_header(
            ethereum_beacon_client::BeaconHeader {
                slot: 3,
                proposer_index: 2,
                parent_root: hex!(
                    "796ea53efb534eab7777809cc5ee2d84e7f25024b9d0c4d7e5bcaab657e4bdbd"
                )
                .into(),
                state_root: hex!(
                    "ba3ff080912be5c9c158b2e962c1b39a91bc0615762ba6fa2ecacafa94e9ae0a"
                )
                .into(),
                body_root: hex!(
                    "a18d7fcefbb74a177c959160e0ee89c23546482154e6831237710414465dcae5"
                )
                .into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("7d42595818709e805dd2fa710a2d2c1f62576ef1ab7273941ac9130fb94b91f7")
        );
    }

    #[test]
    pub fn test_hash_tree_root_beacon_header_2() {
        let hash_root = merklization::hash_tree_root_beacon_header(
            ethereum_beacon_client::BeaconHeader {
                slot: 3476424,
                proposer_index: 314905,
                parent_root: hex!(
                    "c069d7b49cffd2b815b0fb8007eb9ca91202ea548df6f3db60000f29b2489f28"
                )
                .into(),
                state_root: hex!(
                    "444d293e4533501ee508ad608783a7d677c3c566f001313e8a02ce08adf590a3"
                )
                .into(),
                body_root: hex!(
                    "6508a0241047f21ba88f05d05b15534156ab6a6f8e029a9a5423da429834e04a"
                )
                .into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("0aa41166ff01e58e111ac8c42309a738ab453cf8d7285ed8477b1c484acb123e")
        );
    }

    #[test]
    pub fn test_hash_tree_root_sync_committee() {
        let hash_root = merklization::hash_tree_root_sync_committee(
            ethereum_beacon_client::SyncCommittee { 
                pubkeys: vec![
                    ethereum_beacon_client::PublicKey(hex!("592ad40fcec5c0e70f4d6663e3b480e181db52820f69878e3153fb6532eb493d20818d0a5db416df916d4f026f40c713").into()),
                    ethereum_beacon_client::PublicKey(hex!("fd9697146d92b66331f5e4f0a8e40805f39d3dd3480b0f825b94c455036d3d9eff40267d1e1768435079e5cead9ee88b").into()),
                    ethereum_beacon_client::PublicKey(hex!("9174ef2d8f23190c4e7ff6da77b715e2f80e55ef58c48f980fff6b1a363ac36e8e03e626ecf066443ecff3b5b8b09603").into()),
                    ethereum_beacon_client::PublicKey(hex!("7374240fe290230714325d3e6686c91ad79417cb4b170f00479b5d37e2c46607d8dc39b851141b87a5008d90938e82e7").into()),
                    ethereum_beacon_client::PublicKey(hex!("d5c0166b30874667ddd825fff632f1af95b58c0207e04ff0f657d371dd04b1d22127bd22e6c5fbd531271dd192c5e3af").into()),
                    ethereum_beacon_client::PublicKey(hex!("594c0812a81867ca44d55a45a33d33be1d63feb15bd438b252b63bc6f4f3a89ebf5759827a1f8757f2be85f9c25a5bce").into()),
                    ethereum_beacon_client::PublicKey(hex!("82956ee4399ef8cb97298c2c250d697154e24b884a48614d12e624502d0fa3edbbf5221e2f4681f846080195a9a85996").into()),
                    ethereum_beacon_client::PublicKey(hex!("7b865684ff738da3ba8b6a6927fd2a3dea1bb3ab42416733136426985fdd973fc80232c8f8ff7c303a010a221e1f9339").into()),
                    ethereum_beacon_client::PublicKey(hex!("e30df7b9dc4d666dc2073cc9b682a81ea1e0dc09e96b151ad36074878a37eb0bf5c0d9942d4cece1dd4b32d2399a5ddd").into()),
                    ethereum_beacon_client::PublicKey(hex!("54d569acbbe1d06735b5b727029bf08fd2aed8322888db7a6930f9b827a2a0ac57a940a7adbba5a1f9d12fac5c33c265").into()),
                    ethereum_beacon_client::PublicKey(hex!("aef15f3d7cd04342d1c5ede2cb7a51ae1bb9d4db55babce4b6fa097d97c61d6b9e02fee49d2d4b7741f469a38d7c46d1").into()),
                    ethereum_beacon_client::PublicKey(hex!("d56544f0397c0f92e4f31237e780deb6b6cec0e6a0c8dba06a91e06b8bff2d2603342b2cf79fc54f0c47dcc9ccd6064b").into()),
                    ethereum_beacon_client::PublicKey(hex!("1e89b0b766d0b04de35423331d8928f5cab2e94e63c843fb7ef0641d1741e53312f7911566e715e1e44f66b9e24e2128").into()),
                    ethereum_beacon_client::PublicKey(hex!("6bfdd061268631a959a57eb6eeae7e10954c8aec58097c3f259f6ada84ffb6208eb1f9f490fcdfbfbd28d3c9cbef046e").into()),
                    ethereum_beacon_client::PublicKey(hex!("dcd844cf90f873b94f6954fb348d75a5ab862f068501fb8e89d069803bf8d071fbf986da257db2c7200f22fadadd2350").into()),
                    ethereum_beacon_client::PublicKey(hex!("ad25d61f1e4902f7c66eba8d897ad54f460f9a00ced0846513841b6276b54b1ba2ac0654bee5d66c239250522a3859ee").into()),
                    ethereum_beacon_client::PublicKey(hex!("1dd6f1c0ff54f1cabb4be851201073295e6adcaada87268f5e2afe9a8bfb78321b6b3b4562884a74f0b5f9083ceb33ce").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c27cce73f31bfe20c96f3f4865c4a0792ccdebadb501accc3ee4d43d7f18c0387cf08eb0669f8e736e1ce71fa0fad5d").into()),
                    ethereum_beacon_client::PublicKey(hex!("0f419b8494f7d0a2bfcb802c8cb0453fae02962a923817516ff30dee3342966040cc47463328638b894c18d2116593aa").into()),
                    ethereum_beacon_client::PublicKey(hex!("3c9ff7a14e6cee517bffd523fe1bce809c9b92f14f5b68120d630ddc76033d56007dcb785c336616421b3418d220e9d6").into()),
                    ethereum_beacon_client::PublicKey(hex!("67d7a8b53c1095d5c82a686e67457e88e19aabac3852279d4e5e21f77bc61a9885dc922bd44571eea3905d27894c2b0a").into()),
                    ethereum_beacon_client::PublicKey(hex!("2e6215e87002db22be50bd9aeb85f4219e3bb5dc2d136b1c8cb732934b615095630909ae8362fca32fe74e17a93bc84c").into()),
                    ethereum_beacon_client::PublicKey(hex!("775425b7dd5308588a1d79e5b016946054c4d2977ef1a0ad54529ee34ba9b9a707632380c5499c6f42be0e11ae91f047").into()),
                    ethereum_beacon_client::PublicKey(hex!("41005eab7e00694116bcdcd479a5257e7b6f8ac992ef36fbd0b575cdce58ec45ec7d5c7d66fb3540daaa8f775442cc5c").into()),
                    ethereum_beacon_client::PublicKey(hex!("33c01eb38a6e1514b636b6bbe91a67c0889e3bb47cd8ed5082b1ff0460565cd3343991e4ffeddc25d8814191501ecb94").into()),
                    ethereum_beacon_client::PublicKey(hex!("69b1da814d330416ec0c0f54b8a24672fafa5bc64b8916da774a56aa8c9e68f69eba9bf6618934fcb35cf86773305d7e").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e53f3a4a02d7f41db7c41258b9a20e9b90cf7c49e9e23a7f28f9de697a758926fc06f05d7d2d11fb9ec3d6c178b175e").into()),
                    ethereum_beacon_client::PublicKey(hex!("66fcc2ea21cf64f61be9a8ba4f95c99a11d5e184a3b292311aa01948694a85b3824f9ad51eaf2f5a3111a1593e8ab169").into()),
                    ethereum_beacon_client::PublicKey(hex!("aa1d785d10c58c8acacc9282948ecc6837687eef3de96ab05d3eea8e58d0f791362cbc63e217a1c8ff3c97430ff709fb").into()),
                    ethereum_beacon_client::PublicKey(hex!("a3dccd49dc50279f425cc26c1c53244eacb2d3030999ca967d0b75d23b31d1dc57816387155a84eaf5b9ce6dc85a2411").into()),
                    ethereum_beacon_client::PublicKey(hex!("bbee48cb2ac01e8e78a992ad3ad3e771ee56079c88d747a35001c5a17bb865e524b76bca8970264fc6a3b31cc42d7223").into()),
                    ethereum_beacon_client::PublicKey(hex!("881d43305e064c3b91e7ebd23f792eb300f44bdadd59ed4c318d1c1c43f3c6e19f1a480ce869902a07da86abe15f6409").into()),
                    ethereum_beacon_client::PublicKey(hex!("59211736fa864d763f33b79609c42bb7c009f7563ff0262571acc68d93628a5db70bf0902dfa2586a383f06c9f6fb1f7").into()),
                    ethereum_beacon_client::PublicKey(hex!("3e91b39420a2d698c427cb47973d9def12be2829c799c75f75d8eb23aa0f9a1564a978fcb0f6b3d6d97bd2f3c49ea068").into()),
                    ethereum_beacon_client::PublicKey(hex!("34456b26307fdeccb0596d87967a13c6520044954075893f20e206c11f1fef917802df99740eff63b4f95fb03678175d").into()),
                    ethereum_beacon_client::PublicKey(hex!("46b33cf4e60e94c62f0d60d19b6be1b3c5b66543c2f0046f342b3a411d45ac6ed5511de898c456ec0d7ab95ad38415c5").into()),
                    ethereum_beacon_client::PublicKey(hex!("f7275fc4e89be58d8f4191bb0163c0be0739a46e48a1287f11ac5f1a1078e91529d771f320fa39ba6ce921805347c4b0").into()),
                    ethereum_beacon_client::PublicKey(hex!("05da2f1a0b043f495ecc34694d6980bcd3140f6b012b1855edf8409c0485573c32f4d5cf84d6b924f74e0c4614e26191").into()),
                    ethereum_beacon_client::PublicKey(hex!("f68f046604a0e4fb66457bf1aefe8366e187f58403de9d2dafe649ee0b94a88f0605df07db572142899c1950ef19b611").into()),
                    ethereum_beacon_client::PublicKey(hex!("5fdd4c89e6750e7f9be4db00a67af0298574bcb1262e20baaba20d9876af9384d0672a278e7a592509f6aaa68d6ea39c").into()),
                    ethereum_beacon_client::PublicKey(hex!("d160e657ded2ed693ba0aeceafce00a45b6f6918ecb37559533910db5aa2dbd7a4ca2d00f8f500b4bc3fdf8ff28a00f7").into()),
                    ethereum_beacon_client::PublicKey(hex!("c80f5c3907a3aeb71834f15b9142230adf0dfa0346974dee0efe4fe126304d8f40e731d17de257c5ded5337034d4e157").into()),
                    ethereum_beacon_client::PublicKey(hex!("774b9a91584563e453be5f389a7c803be33c9043b72860107a36917fd086605be53b04ca506ce4201272ca67326f133d").into()),
                    ethereum_beacon_client::PublicKey(hex!("35a346d536f08a6814d4eaa516ee6d76790c1d441a134bf6ed90efa634acf805b5e7f6f3aa21036ba6e73ffa9d1fd71d").into()),
                    ethereum_beacon_client::PublicKey(hex!("39748096a77d9632237541184c8b5470feb99d823c3f76d297fd47d9107a0d99f00ceae4ff8d563d8184e1aae33f0037").into()),
                    ethereum_beacon_client::PublicKey(hex!("833f50691636c04cf0568166ca2b69a2e71265b6454a7cc2542a2942614964672c2813b2c02b1ae50078f9e87d7dd2b2").into()),
                    ethereum_beacon_client::PublicKey(hex!("5fb2185421e6bbe56bfd27f3adc10bce06f6374028ecfe4c59b0f21a2697565e7a40399e19886e3b646a1a74bc34f0f4").into()),
                    ethereum_beacon_client::PublicKey(hex!("8926ca48be3126257a5fecc6efdb623b70fb0a7fb8086351f877849bb868f0c69a0ca8f1cfb97a67bd51a479dde4a1dc").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a977485281b39819ae110aaf19e825af8621dc7c2565417c22e8285840974eeb6de83c23102996bca3e419b00ca73ab").into()),
                    ethereum_beacon_client::PublicKey(hex!("26294201dcbfea5675180a5d5baeda262e0fe10c971842268224e9df1572d21975a210b0cc33a46222519a40441dad9c").into()),
                    ethereum_beacon_client::PublicKey(hex!("e90202aa6ac88c6a1f3ec2ab9bc2267b10fc3a5766e38664ad0a8b6328ddbfd5738cb3e884a97527696c76d59c70a4f9").into()),
                    ethereum_beacon_client::PublicKey(hex!("d1aa645377c0b2de860f36aae0f8122cc7756ad92bcc3b0235cc6b89704b7d5c01c1584f6681137f92de2403918a7b8a").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ed05586aaea0dbecf5a7ea958cea78ad2ea01ed27fcd6d1202129f26c2d189c6525b575de4256fcd6ce0da575299ac4").into()),
                    ethereum_beacon_client::PublicKey(hex!("3ec65e0317b32719a41798302a3b64a184bdf1b56d3315aa9d8bbc4b4ee60d8c26fc7e2d4e384873263704e462ee82c0").into()),
                    ethereum_beacon_client::PublicKey(hex!("150aa17e6997bff5b0f630d1558e24c4276f57f2a286c0adef5b6e5b04eb2f7f490e3c05be745ac75e28e9938189eda7").into()),
                    ethereum_beacon_client::PublicKey(hex!("7817e17b91f98c23c10c193679a379c339dcec598a0866908341e20ba08e558f49973c40e33d8b189fb75a93f7a11a5f").into()),
                    ethereum_beacon_client::PublicKey(hex!("92a239b186bc2f64e17d4e6fa9e1e7e72d23e0da131e1464aa364ad20829dd95483e2c3ec8a66c937130e5769fc7969c").into()),
                    ethereum_beacon_client::PublicKey(hex!("c030e963a0e9cd438f3f342dd35bddb98756d9ad1b383b34f3984b4e45e0ad511281ec57b876d2aeda07306ad9148e1d").into()),
                    ethereum_beacon_client::PublicKey(hex!("2df883b5ea2fc447d970d37ab1ed00afc05877087f827c6b0972d91e0bb6e4b2e31fa0d7a3433c5e49259278793196a4").into()),
                    ethereum_beacon_client::PublicKey(hex!("3e9179efb4b7db1fc7c3c14a8ef08c823c724ec9c75d63dc4d3670b062a288f24d608de490f5a090584bae36ef787742").into()),
                    ethereum_beacon_client::PublicKey(hex!("4561a292b2d11d365941221cf4b35a6c0e01cd20355db2cb634d57afa570b15ca03d21a9ff313522d40fcd3e7c093a22").into()),
                    ethereum_beacon_client::PublicKey(hex!("a7b1c4ecf19324d3579c43217173b1d6c72c5efd572a9fdf468a959cad0688c161025468702751ac31fe40bc3d8c09ab").into()),
                    ethereum_beacon_client::PublicKey(hex!("b7948546b20c78627d5315341ec1dedc75c1fc8c8ded9a11b6c43dd089bb3c4e4b4a01d5c052466114ba03b35852c5be").into()),
                    ethereum_beacon_client::PublicKey(hex!("3d76765417d16c953b267e094560a639bf00089b6ab74b7a35c8dd97994553c6ccb7feb1623023d1953264b79bfa7eff").into()),
                    ethereum_beacon_client::PublicKey(hex!("e9021d4e9b8b82de6714155a7aebe86bbec9afe96d59c34b8f0da7d3ae26b7710d317267145ab3f9e4b6de1e9a9a1ba2").into()),
                    ethereum_beacon_client::PublicKey(hex!("451f7ade7504aad289faa5d47aa4f3961b511ad66fc29c86a175cd8b86550cfa80f461ba1d07087545f52f5c26796398").into()),
                    ethereum_beacon_client::PublicKey(hex!("ef5a4dff11da3106b57f545bffa7903f5a76e6f11e3b7373e4dd9cf8692985d280ac799d7bbff0b7e1209b2a289117f8").into()),
                    ethereum_beacon_client::PublicKey(hex!("0c9ce4d6a5e7c62ba628861b8b2a125d4ddebdb2643c89e47bc7a74e1176d6c53347d81fa9cb610de72d7382bc42cbc9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9be385450781312e856fcdd2d5f33cfd9f41e0139a7c1fff20066e7a44da4d98bff0a20507677ae0aab046577a20b258").into()),
                    ethereum_beacon_client::PublicKey(hex!("dfc86865deca183ddb0a3a5e239b4a96a5bf0dfc9d0238654d9f98aa499cb24ee27ed53c8a330823e38915340896cb19").into()),
                    ethereum_beacon_client::PublicKey(hex!("12611949e77eae716e210fda053c4581ffd8c746f71378a088a4d9098900de1b403798a17e4ffb99e7c09331b766954a").into()),
                    ethereum_beacon_client::PublicKey(hex!("03c33353b56c37fd5d7ecb9a2621e806c78f1aeb81e48a56a7e80c4ac13bfff3817a457a2ef7be88d84a3b7643049b84").into()),
                    ethereum_beacon_client::PublicKey(hex!("6b346a9bd9f64f3648aaf5667adca9d58297612563a9de50d65cb2327b13eb1447b4df2914391585a51cd1006d6f8b9a").into()),
                    ethereum_beacon_client::PublicKey(hex!("54711b443b7269027f5d8785b15ad00330648c0af885d4e55c57e805ed0a58014d6a904f15168c0845960c88e6484390").into()),
                    ethereum_beacon_client::PublicKey(hex!("6e465cb124b6c7b22a6fc3cecfa6285286b87499fd68962c179530abfb7633b8d277aef04a5da5a67244ac7d221454a6").into()),
                    ethereum_beacon_client::PublicKey(hex!("40d43031189d535ebad1f67244e819a472947cd90f7d9ea5ad8713955d7757a04363045e12d650a0537b57f64c06265b").into()),
                    ethereum_beacon_client::PublicKey(hex!("aae725a2e72b93105c0dfa90bec925a9342cb385c98cd55bd43d310b30df2e6e3c68316a8684b9027ecfa074259aedb5").into()),
                    ethereum_beacon_client::PublicKey(hex!("6c16482741c737d33381fd406b119f5ffeaa56c2afb11feb22af75179aaa093349eef9ecc84fc2a3f8d803bb9dc49343").into()),
                    ethereum_beacon_client::PublicKey(hex!("e9df20640774b6edd1d0af5ca87fa11e11c9dbc4c723d4380e97350e1b2080871e34961e3bf1a966b4271251c4fae4c4").into()),
                    ethereum_beacon_client::PublicKey(hex!("c2db5407eb2f083c176594527266189ac064df732e02f209f5bf5cb8e392ae0ff1f9e2dbca5e938807dfa1c91dd5d278").into()),
                    ethereum_beacon_client::PublicKey(hex!("06ca35839eacbddf3141777b627d7f0d8e75a0f0ee50d30c639dbc0e86275e561566af9ff4af282e24fe2ce12bdb1293").into()),
                    ethereum_beacon_client::PublicKey(hex!("f2b1596c0b5095eea22d3a1e248f74ec4cc1d4ab5323a53f04a454b11bf4cbcbb7d5e8f67d1232ab9ca7a2dc821dea21").into()),
                    ethereum_beacon_client::PublicKey(hex!("9985a2a35d177203f370d1ac24c1581aa7e46cda8eca5c35e6dd49425f1d691c4c6820ddb413f93067ea382f532631d3").into()),
                    ethereum_beacon_client::PublicKey(hex!("29d11321174d8f1f5ecb3d28bfcdd0506bccfb447892d3920bb55ec7fa087263cb5b4d322d40cb63833cf125398535a2").into()),
                    ethereum_beacon_client::PublicKey(hex!("d6edd7af27555832b630f4c7403bf7ee83200b9fc91ee1acb15d32d9567a1c32ccf2205589f64bcd641c7d83af96d1c2").into()),
                    ethereum_beacon_client::PublicKey(hex!("9549d61156db24fa2fec8178dda6e000be4dc4d0402b6e59739ccc38a3dc0a5e58dcc0c701cd439d100553ce8a00b8db").into()),
                    ethereum_beacon_client::PublicKey(hex!("99c3db60f2ea4d0237583f4e1f5051c31b0b998c8a3983e31abc498c76449234b608a2210cdbb2af57f435f720d3ff21").into()),
                    ethereum_beacon_client::PublicKey(hex!("3a30a77d28767ca707aed12587f9c6850c3ba15b7881f0b78ca67639b6e08c9f74196d3dc91d41a16aa404f401bde9d8").into()),
                    ethereum_beacon_client::PublicKey(hex!("aaf332dda99e8734a0d6d25092553a053da334eaaa4a1c27cf4eb11ee73052dcab482dc4ede1151909d40a26053ad49c").into()),
                    ethereum_beacon_client::PublicKey(hex!("ffc3623230153c5187b976a9335ac8990280f0ae6299dbccee9936989b9ba254801c8eb2b54991500027c0369479d26f").into()),
                    ethereum_beacon_client::PublicKey(hex!("6d4c0adeda48e24b598b84d9ca0d4b888bd6125ac3973809c4c583284c9ac84a7bf1afc5e0d10b0e37250b25df966205").into()),
                    ethereum_beacon_client::PublicKey(hex!("779270764bd2804d044c4a634e5ff0fcd157e986767a2a16911faaf9324db71be8db672fa3d3e67d726549bfbca2bbe3").into()),
                    ethereum_beacon_client::PublicKey(hex!("84da89873c5e06161f9fbed011be93579ce6543dfb0d848d9450cf1f4c932f82b2661e2c1e28ccc8c85e06a5b1d13704").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b9f404b44c809ce70a964b465e90da3aad04be36cf444f80aaff7adabd36ce6e4b1521f49f7d37493219cbd80ee152c").into()),
                    ethereum_beacon_client::PublicKey(hex!("3c39774228c98051b2e2660d3229746857c006c2d0cee7bd0c8c9c4626fe6c83cc21d7113a7d46c32da00281a9ae71c1").into()),
                    ethereum_beacon_client::PublicKey(hex!("67e903a9f8de3c19d4dd0593aec5efff05d0e1bca96baef18d0ae0ce43acfdb06ed50e5fc2569bef166cea66b0ddc1c1").into()),
                    ethereum_beacon_client::PublicKey(hex!("5238b6a910f190c29c2de8251cc9b1093e533185b21f83a7354823ec05716ccd39198c225f05ec5a86f739ba3381b843").into()),
                    ethereum_beacon_client::PublicKey(hex!("c0296847fe8eab3547971b2381fa96e4f12c9aa31c757421bb32081a250aa9ded1ea4e9a0632c01144211c52277fc3f5").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c212df6d5613ff8412acb4ee33b03f3a82c084b5ef7bb36df27f9a2af9c93bb18203175c4726db1a2aa28c2e1a3a501").into()),
                    ethereum_beacon_client::PublicKey(hex!("6914ba0678a28189fdc6f008ea5050392d1221f0456833dee8033bc327838810077e8a6a8dc924717396b6fa3c2a1072").into()),
                    ethereum_beacon_client::PublicKey(hex!("daafc547bf228ab800104aa5c3bca8f7e4ce89f2d4ba4e54c2c0b3e4dd26ac505550db8106f97058e8fca5bbbcfa09ce").into()),
                    ethereum_beacon_client::PublicKey(hex!("abc0e3a7b44c2643d00328a760a002d280e022ccb390694978822f9c4b426497fdf0de9f12696d9428d2d0bc90c25a55").into()),
                    ethereum_beacon_client::PublicKey(hex!("71e0902422bc35043cfa16869b2136df6f71459d9cc92739ddd750e9f3aeb9e5cdad9943a4885b4b6c3f54fe536c0c27").into()),
                    ethereum_beacon_client::PublicKey(hex!("20988e1b6b1440e5252a289666d60ded7e0cba4f4f8ba0092a7930a0a1c55da458e7ef86dc52c4713a0fc9cae96b053b").into()),
                    ethereum_beacon_client::PublicKey(hex!("4d55dea5157a5eb5abb901f211b826643fa002a1307f6409b77c49649b2802b5b9eb2e5f168a0359b31efdc4295eb40d").into()),
                    ethereum_beacon_client::PublicKey(hex!("14eabe5536492eda3c9e43f2895216aa16fbe63b99efd81614e97d5b359637d555e514cfafc620a52134f53cdd4fd72f").into()),
                    ethereum_beacon_client::PublicKey(hex!("62d1c771b75b6514cb1dc7cc2bfd7f9629be147ad24cdefe615361fbb20d197994dc505816b01294c023412767a371f9").into()),
                    ethereum_beacon_client::PublicKey(hex!("ddc91125bf98f9db27cd1ea3bac1a1cd5477ab1054252fefb95d41dc9e0c2f5b194bce3085d910860ad9f32be964ca47").into()),
                    ethereum_beacon_client::PublicKey(hex!("457a01c1596df8407e7619fd3b9b7e1e7743bbc0144bb514cc6934ac98081d6e4269b23ac40989ecedebcf3b010f6888").into()),
                    ethereum_beacon_client::PublicKey(hex!("9954a816a68b2af847367844790fd4d7460d2bb6130cea83f7c8798c92aa27a5e104d20e269fb48823c66f6951e96e38").into()),
                    ethereum_beacon_client::PublicKey(hex!("47fa6c841bfcd44a0368f25b4a8faefc37cfc9a749800f0888a99d9c490944150fe23ab6290352efbccec55561b2b96d").into()),
                    ethereum_beacon_client::PublicKey(hex!("99e0ab54807fc7e848ce2f02f0b57ba752761566836abd95cd8806f83623260945c30316b712f270d3cfb63faa970d7e").into()),
                    ethereum_beacon_client::PublicKey(hex!("a9f647d23fc073c845d36f840a7747a91a8d7fe93c727292527280e2ce91668f278d0295291a658886b47fccecc892d8").into()),
                    ethereum_beacon_client::PublicKey(hex!("ae228cbd2c63bf265dd7af534114ccc0fa180b6b9fd6f5e6d69f3aa266b876139d27df8bc06e19e6850988fd43b223ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("7a442b6fddd08efdd2b22074b2daa8978200c960e766dfab3891f4d0f7c8a80e4c9aefdb0fa9acdd4ef57e700a1e17d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("98c319e76300e2df833003b96293dea393bda83cbbfabf211ca9e74c2d037e4b592893a915eef4b3945c57b155cacdcd").into()),
                    ethereum_beacon_client::PublicKey(hex!("ee9549cfd64bfb3c14054c115f51c9edfa11aef90a27a05d698dcf09f5fbcc61d24469e64640ae272ac176a8400150d3").into()),
                    ethereum_beacon_client::PublicKey(hex!("40e6e84b13c80f7c9152f7b217dbf10aba2c83f95a457dcd29d2e775cdd9d3c8d2978ac5d10c6b42bb3bda636c5cb268").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b47a82f1977a0e4f4f6f72f1763204cb8600ca49a6d2d8471ac9bb24b7fba58cb008bca19acaa9df4cb054bf5744461").into()),
                    ethereum_beacon_client::PublicKey(hex!("b934915b598c4ce455ed2db152eada53a69aae79f180c019fc4b111adc25ea6066f0a7d64dd7fcd21cd991266b8ded3f").into()),
                    ethereum_beacon_client::PublicKey(hex!("3665f08c33894e0c486099387a341addff19434e5d994b0fc3cdaa3132cf7deedabb9cd412a4d2ad23193738f828ee5a").into()),
                    ethereum_beacon_client::PublicKey(hex!("70d43455c1979c7ef47a2af28cdd3cbfeec99381ff3b0b265ab0ac2822e76fcb3b8d150918a2844dd4ec7f2e5e3e3d6e").into()),
                    ethereum_beacon_client::PublicKey(hex!("dc4a7161925eb9d638abf47f7c52f11d6c5a4c0b6bd22acdedc90fa1aa4d38d8d2e3135fa9336047881245e6dfc02ffd").into()),
                    ethereum_beacon_client::PublicKey(hex!("74e3a9222eaeda103b4e258de5fb904712ec32a7508464445c3b850a8faae3e7baf49e9d94bbce113a324bb7326259ae").into()),
                    ethereum_beacon_client::PublicKey(hex!("d1c4f1797492b5174b1ce79cf0e35dedced7925f3ce2f63e8817c280b2ab1387a60fc182fc063e12f0b3c186b4875b71").into()),
                    ethereum_beacon_client::PublicKey(hex!("5efa696e2ad91bf24134c6791ea91e32a17536f6d9b724906a6d82356b1ca2d288830300cdc3c4fd713e253dfaf370d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("4fa67acfdeff8cb4c7773ff11dc106b1225e4ec44bdb8857d402366202df51e6049b6d9a3a13aa9b13dabc9b2fba9c6b").into()),
                    ethereum_beacon_client::PublicKey(hex!("b220a5224e453335f170cfe7dc8eb1e86dd1746dfa8cde4abc115621f8042ef789ba08ac0b460ae58c38df5523b83cb2").into()),
                    ethereum_beacon_client::PublicKey(hex!("eccf1b85afbfb973aba29c817e18041b54e8ca72d663a0c46c074e0138a2bb73c192ff49df3a3dbc7a3d98f322984aed").into()),
                    ethereum_beacon_client::PublicKey(hex!("83e87639baf3593a6f1c095a362d1b103190a9f31f1f008619259c148de4cdfc02b14490fd7d5db36555c74caf81d48d").into()),
                    ethereum_beacon_client::PublicKey(hex!("66cd922fd2adcf753d0a9f7101dc80fdd5e47355309fba4c837778992c5cdccb6128674a11b2f1fb12ed9114a7504197").into()),
                    ethereum_beacon_client::PublicKey(hex!("582b88e41895a018714532430022de480ef109055200d17263c7e452804351953f4a2d10082f61a7532ac124a90e764a").into()),
                    ethereum_beacon_client::PublicKey(hex!("b05174dcd3bd22b82b68c5e43ddc51e5cb0eb491801654a8701758a6f25075803a9e83bc06f29539b129817aa1124f68").into()),
                    ethereum_beacon_client::PublicKey(hex!("a270ce53f2101193cebf79564da6762881c7ab76f5b2e51adccbc64875c0c276e6eddaa701dcdf9d5fc508ed7d144b13").into()),
                    ethereum_beacon_client::PublicKey(hex!("9da80283fcf4ae1a382efe93f4a0251b2f1e9c4cecced6ff82e489e5b795551e5551e90ad84513872713ab99e97569f4").into()),
                    ethereum_beacon_client::PublicKey(hex!("d2627b8cea52cf7efc4bfee5e0aeccd3f5f74e9d96b827b7ef01f56be0a3ea155bcf39d02b5994cd239c390ff3ddd12a").into()),
                    ethereum_beacon_client::PublicKey(hex!("691423de74e583d34ced94b0cb7e9d90eb0badc034b4bd57b2522a05c584560de877c2713e960cface373250795efbb1").into()),
                    ethereum_beacon_client::PublicKey(hex!("b900d27e26a4ac2cc592afe43b55b127f7e212cb9cc271bcf773a14b9701831f2d26937230c13e54b8a26792b38ef852").into()),
                    ethereum_beacon_client::PublicKey(hex!("4c3cfd02e0a08d23d27399c128ae13aef5c3d6de95a01df35ce650f3381315c426c7d55d74ee29028902ec830e7914a8").into()),
                    ethereum_beacon_client::PublicKey(hex!("85044d41872397360a07c728cc23c6608fb8659556b909881b4d4cbc0855c8086236ed557a2d29f20f10f93eaa9e853b").into()),
                    ethereum_beacon_client::PublicKey(hex!("83f7cc9ce77ac055459febe75e112c1977eb0bbe10cee86660f61b2ffe2f21b88ac819bf5a9f75025e1efef5f535059b").into()),
                    ethereum_beacon_client::PublicKey(hex!("03515113d9ea4ff33ca8d39f245c537d4e208ca8051233f2e8e6310b11113c041ec6b4cb056a7c6719535dc90a0d1a51").into()),
                    ethereum_beacon_client::PublicKey(hex!("dd60a5aebff91b751904af5d41a9a7a889d056d6d3aa5d9a51cb53f185b55b9d360612062d981feda2c49d990a3b7ed7").into()),
                    ethereum_beacon_client::PublicKey(hex!("8632b93ea8088006f21b819c6b37d62f5de8734b36f890216eb4e64a03b317a26712b649e0b1a6707144918dbb2d1992").into()),
                    ethereum_beacon_client::PublicKey(hex!("84c7925a563467db0bd502db87ec182fb01c3bb4981eaf933dcd68bbc3ab4a088a956f31990aef5371d038fe68c57ed4").into()),
                    ethereum_beacon_client::PublicKey(hex!("f03add2f7f65b9299129a6725ebbe8a8f80b2256bb3984fc574cd970a58f7fef3eb12d816ba1a64896faf1cbc2dc4b40").into()),
                    ethereum_beacon_client::PublicKey(hex!("792f40be7464ada3b8283405993ddd5c374df5acdc1810da4992c69507d4c19d6aa3ebbe5511c300793dead4f2981f85").into()),
                    ethereum_beacon_client::PublicKey(hex!("27a7a587ff7d68c1f6e5a6e51913aeed47f007da3d6914516a8b1cea931a613eebf6e251a1ab96ae5d49de2208f92f6d").into()),
                    ethereum_beacon_client::PublicKey(hex!("7a1f80834c47b2b128197fcf0439b540b3dfa3f6a6239af5566f7d6712e36aac04f7b1c1c401d0d68bf261a801e57b43").into()),
                    ethereum_beacon_client::PublicKey(hex!("916006e47fd50e504916cf847e4355c4b209b4173f73a0f0d73da3d7cd3ea01ac5c0702a2e505a56f09ffab0c7afe77d").into()),
                    ethereum_beacon_client::PublicKey(hex!("07ee37c12280ad367d8b3e125d59cd4e223bd030182fa6ef655f38cc1d126603d228d10b0d0725d9238f62674c72b783").into()),
                    ethereum_beacon_client::PublicKey(hex!("99e840e2b15ea4c8a0af9df164ffaecdd767e0ca3b74b16cbe68efa10c1eb427b13af36880ac3fb2cd43eb5ac4c0fee0").into()),
                    ethereum_beacon_client::PublicKey(hex!("be333e4367358a301c56106ecf7f73a195d43a6d3a617382f02c9fa76a686a52859226b40a7294c11d79dbfb538fdb1d").into()),
                    ethereum_beacon_client::PublicKey(hex!("eecd34ffd24036c47d3730ad7db0419c4bb068db585d78fbd2abc1082302309cfd2a63a52b889be1a4c4a3b706b69313").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e377a7c137ee0941f12c283f7219d0366fced6ad4487f232ca1e0259e23c6529a3efc6e407009be0e8f9e422a2d21f7").into()),
                    ethereum_beacon_client::PublicKey(hex!("d47f9bca0f2e9d9c630475b61fdb3eb166d7d13fde030265ef7111f882a31e60c9af2038c82896ba66db8d0dafe59342").into()),
                    ethereum_beacon_client::PublicKey(hex!("b23ff52d6a2581fe6666ebaa9e974720fa61aa76339348439cf6b438501ada02011e359920a332f4ca313aa9d6793e33").into()),
                    ethereum_beacon_client::PublicKey(hex!("a6cfedbeddecd02af2adccc34388e903bc902cafcb69250c35a72cc23e6b6fdd354f025374939aaf83b829a9c6a7d630").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab01e7831103a563c6c71ff62bf9ae6f12018f5e13a9c77bdc723e7e6c00273a9c06ce12e7376705c346d6977777b8f8").into()),
                    ethereum_beacon_client::PublicKey(hex!("97ad201ddd583922b820d75257194d9bed542563266478d1ffd4258714e4fe0a579bdcdd869c3d1609059817dcbdc97b").into()),
                    ethereum_beacon_client::PublicKey(hex!("7dc8dc3502d8c806467b11604c358f2f4a8b373089dc8d747730a1b404c4632602496746ca009a1933571515db72fc37").into()),
                    ethereum_beacon_client::PublicKey(hex!("98ee3f9ea03840eab045c327afc9e5c6ca29252e15e430425f806c5107410c17f367a3318481f15541f39f0c8830c2f5").into()),
                    ethereum_beacon_client::PublicKey(hex!("6631a1b7f57469accf1d6f62ebf71101b0c63086b60eaecd8a4cc610fe6d147b36568cd43514cb00a939d3e3e9f5f64c").into()),
                    ethereum_beacon_client::PublicKey(hex!("c04c762b6ca4b00710620a0a823f0d45f0b70de6de17d11f17197a894912f0c2472b15a4a6d77b0da150f177fa671d43").into()),
                    ethereum_beacon_client::PublicKey(hex!("59a29648bd074ba0b58b1be1b6e624c39108818f24a2c205654846c8a5deaa5dd8223528287660c44380c4577464d989").into()),
                    ethereum_beacon_client::PublicKey(hex!("f2d9ed5e331ac7a154da9a9fb8afae24128834e694691ccc045aa6c7626a34b5cb5b27a4febaf65e5c6e5a4ff01cb761").into()),
                    ethereum_beacon_client::PublicKey(hex!("97252851d18da63138229bf5579c403992e70787b184c00a92c52715d1c2ece7ad8dbff4e0395bcc59af34b90c1a55cb").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a129cdf6a308aa8945aa5976d37d694653e8a8b5e9c1045e9b2998bb95985de384844fa4c93f0e293ac02d2c0fea10b").into()),
                    ethereum_beacon_client::PublicKey(hex!("39ae46323e61ce8fc7c00a5a1dea76dab333e45e28cda9ddf4b3f070a0788554edd233f6f9698cdb789978bf6d6a1c50").into()),
                    ethereum_beacon_client::PublicKey(hex!("005e9f18435fd8619cd7b172fd9cd5922333e050f6c6e225d6b9d4f7f8758cf464a2aaaf659da1b99115be05fa2d712a").into()),
                    ethereum_beacon_client::PublicKey(hex!("5d18ed5c1b7873925bf6f977f311b9106ff21080ec9589ead61105c3056cddf654bec257ff1f52fa60bb55c15a7684a3").into()),
                    ethereum_beacon_client::PublicKey(hex!("50713a00b7143befadbd9c13eda936246c7518fb6eb1a180a3f198d1b671ff2ba0daa577db1f8145b8517a6ee992dd11").into()),
                    ethereum_beacon_client::PublicKey(hex!("d55f23269ee3df4f0c809bf0a4492b0ee45f8e54ef61aad3662ce835663064d52cdc8b0428c1d02657271b19b588b770").into()),
                    ethereum_beacon_client::PublicKey(hex!("8f4bc64bc204ad66dcf4d217fd2298cca01198c6ab187604c0142b83ee4b96380d240f95b44142b09c101fcd7719b932").into()),
                    ethereum_beacon_client::PublicKey(hex!("f2eff3dedf0fc5b96fa92335a273fbe1380aa14fb305d3ddecfb45c9d1205341c5247f1c05d01ec74dc518b2faa42cc2").into()),
                    ethereum_beacon_client::PublicKey(hex!("506caeaedee1bafaac3184596f7a3048671c48d2c000784fce0e5414785be22e9cda50f91b66234e8709a0158d8a125a").into()),
                    ethereum_beacon_client::PublicKey(hex!("a66df170d6df7a816586c38b63fa44e1d2d342cfd479b3d98e3dfa0e51302202005e52d69ea498f5b38697fc21335e47").into()),
                    ethereum_beacon_client::PublicKey(hex!("a8efb7f4c8e7adf628ab6ce6d41aee80037d300a3cafe39381fc0cda4178f31b342833af90cef41dad8ccb6f35c4e918").into()),
                    ethereum_beacon_client::PublicKey(hex!("9f6205e44f238add8772e8512c4a6113ce68668d7d0b5de793b0c8311c7abc8b9d0866c2d8f8468b8b64bed0e8bc5b0a").into()),
                    ethereum_beacon_client::PublicKey(hex!("a105bb8c171fc59f169842e19ca67b65ecf65d49bdafb0c15e6feaab3aab276ab98bdbccae7c5687fc86a810bd696ec3").into()),
                    ethereum_beacon_client::PublicKey(hex!("a62b760282c17398d441b3b28d5a662fb4d422d3f1d19eb32e802e1762e46fd7bdb0249fac79d27a4ffe93210d132836").into()),
                    ethereum_beacon_client::PublicKey(hex!("8cf9bccae19f190d1878500d1fc2b7189185cc7de38432e798634f27e8c5dd4c63913b67c83633789f93ef931948b402").into()),
                    ethereum_beacon_client::PublicKey(hex!("1c0b102f99254eb527951f2e5919b554cd89f9d179db24c0f6fa50eeaaaa1a6e09e2265f836f20c0d01ecb7fba042879").into()),
                    ethereum_beacon_client::PublicKey(hex!("85baee3657309168bd587ee03fbe3098e64dbeb5501f232b7ee8dfa7e194c97e0b07f491c63ab50dcebc26e787c55eaf").into()),
                    ethereum_beacon_client::PublicKey(hex!("990e8094b73cd024c969bcff6579cf384d772309ae5bb67f73ccb8c14fc1772488112d6da06ed1bdc259192047d3c1b1").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5f7fccb1715529b08f0f4bd19e2dee442e05dc034ce83abaf5d3a8b730781fdce5f191b3eec88138a778d54f7eecb0b").into()),
                    ethereum_beacon_client::PublicKey(hex!("5edf0543d2fb0d0da48b6513315324a187ccb121e4dc1cb0e20aed9aeed4c010493dfe6eb29e40a8a1cb2a99d66c5082").into()),
                    ethereum_beacon_client::PublicKey(hex!("8c45c9c4dbc4ab72011de24e77694fed0faaa79d0942c945254bbb9801c6e02cfc34be2a2c7d4e3ed88974da4d4913bb").into()),
                    ethereum_beacon_client::PublicKey(hex!("2ad4a4bd60fe71810354ecfc6596bb49e65968580ceae6ec119ab5bcd40f678c9ae3dcf8514f42bc6eccd19753909da9").into()),
                    ethereum_beacon_client::PublicKey(hex!("5021bf3b9596e9609e20742bf919d6c7c4f08b741516956fd27e3a4ca3e6700fe97836d7912b6296c0bf036255ebaf53").into()),
                    ethereum_beacon_client::PublicKey(hex!("7d11292b615863b3c82f2cdee2504d4e91ad1c67bf0886e577605aa5cbdbc754b7517d97302afbaf797e4b8bf4bfe290").into()),
                    ethereum_beacon_client::PublicKey(hex!("74d036f2fe387100e93ee79ea8b4b925941e97a55f65adef72a5b09d7d7c7d04d8fa00de40355849357a5a794c6e92c3").into()),
                    ethereum_beacon_client::PublicKey(hex!("d369e227e7f80d1c0b5ed8b56906493f400e8df32c27c0948bf880410a01d4c3be9391ffd98bea94084590161fc40e95").into()),
                    ethereum_beacon_client::PublicKey(hex!("baa4425995be82cb626d5a46a581e201e6655567995fb9ecd1eeff3895c424d6540cde53cde94e30542bec8547c5abae").into()),
                    ethereum_beacon_client::PublicKey(hex!("2e72aedc36a9aad2203f1346f2276f0cbe0e6e2c14e2958acd2350a833355a3a487d8fc00bb62c3bb3ad572b8fa4c80a").into()),
                    ethereum_beacon_client::PublicKey(hex!("2cb79e71a08b9c7587412bf3fb0719f104d35758a130ad9c5db5eb963eb9b0ca48b1818f0d2e458d77b7cc1a0c731ed2").into()),
                    ethereum_beacon_client::PublicKey(hex!("0ccda1e12b2ec9b5b51ad7f9b2da7b7b9c5ca3ce1691255632ae139fb20c390934b52dc802e983a175f77f3601ffbabf").into()),
                    ethereum_beacon_client::PublicKey(hex!("cc6a02131d9a559f2cf478f4caa2f9bcbdae303f6cb255d09586ecdfcdd52a6139f78ae9fffc37aac26993fda0602d83").into()),
                    ethereum_beacon_client::PublicKey(hex!("bc4e870034b5e059ab8e1a757dfe24558ad529c31149b6ead6fd660327df23175238e97e5d8ec2e5d4adf3c89dcb20da").into()),
                    ethereum_beacon_client::PublicKey(hex!("5e971ea94ed788af8d9ae4c2800c1861a7a18a1c170d9b9918a5b811ef8d9cfc1523120078aa875c7a8ce97f7d8e0d95").into()),
                    ethereum_beacon_client::PublicKey(hex!("6ae848fe0254343869d0fb87bda7a9b0a8adb01fdac894721905ad8113116abe8f5cd50cc2eb4466c25060fc22285524").into()),
                    ethereum_beacon_client::PublicKey(hex!("2bbb0cd0c14b385077ebf9a4d4ff94bf25a2dddeadfd28665b0bae88b6d1706983b3cd5869cd55a6ed9fb8d05ea7e658").into()),
                    ethereum_beacon_client::PublicKey(hex!("42fe0da184a452ebdb5a5a9eb66aa78c804fcc763272ad38fed2e1e2916a98aec184a1a2f28100ea45e5cd4396e04a04").into()),
                    ethereum_beacon_client::PublicKey(hex!("ed4f8e38168ddf72fcb40185ab825c137bc3f165c045468f22b94ef1d7c544830ec912c0d514fc6888fb6a00b1dd48ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("9d158487f2cfcb3dd2e5fb772e8818a436226584113631b160a5e63f7d8631a1449bb2dd807798c6fc3cbabdafdbe2f8").into()),
                    ethereum_beacon_client::PublicKey(hex!("14cf0527cacc56af5bc9a41d446dc51f12f20ff88862fa93b16fa8472f3834c0879050f3502c98eb46684584758557eb").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b9410630e1b016180aa5aef885464e442d7939a05442afc24139901a3644316c4421d8ec62f84b8bfd80cc4ee07dcdc").into()),
                    ethereum_beacon_client::PublicKey(hex!("c7f7e27d9febe978673bd7407e2b29a8a08200151cb1e4c80306eb2324aea1ff44d84761c748d146aad5affbe7c8dbb2").into()),
                    ethereum_beacon_client::PublicKey(hex!("7d932b16654517f7ae6a24ad2504121a4c2ef42cf06f3c793b50a26cdb25b335dfc35b4239774ba850331ee5083cafb5").into()),
                    ethereum_beacon_client::PublicKey(hex!("bb826f00ef57af37846019ebefe7048743567e66afba9f205a4dd823193b9e83e01eec9e5df9cd95bbe15001e88ed9d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("0f25e8e8e25274a3aa1fd7a075f476fc265be02585f3c4d36730dfa01cb4ec82d0765ed1cbccd8489ef5a4ee1621e84b").into()),
                    ethereum_beacon_client::PublicKey(hex!("589b1616f4a3d3180605bdd2011fee71686f331c6bd25f2a8e533311788b8dd49bb474fbd6df67a80ca6a39276d6b882").into()),
                    ethereum_beacon_client::PublicKey(hex!("7347fa5cd94995bedea7822bbc63dff63dc56435a6262cb311c7bd15393ab12434868b9695d6ae0ed8c1ce8823c542eb").into()),
                    ethereum_beacon_client::PublicKey(hex!("429faa123dcac7087ed3ffdee6bcfd0da9f4581da2b767a11d880b0399cc487b87079e4a435ca580736c2d4507841848").into()),
                    ethereum_beacon_client::PublicKey(hex!("886b8b424f8b73c4cf21e763bcc0bdaf214027778d20c0a7aec33d6df166db959031dad3bb1ae9b8451cce440a66cb1e").into()),
                    ethereum_beacon_client::PublicKey(hex!("da4f352e0fae62db98c4851f0d724e415a199acecc39bf1701237a5fbe0d12957a8f9641780f11905404fb7d2824a2aa").into()),
                    ethereum_beacon_client::PublicKey(hex!("a52c9f919eef29dbd5b7afe6c4b320ae667c598ec96767f4a9d2b98847b561d18906947584377b5390ddd521fa39c850").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab0e0ce404986c8e021cb294603d95633dfe9dfdf1cb64ade41ac1ea1c7d4dc257502987543285fbc39aac05554ac947").into()),
                    ethereum_beacon_client::PublicKey(hex!("ea7032ce742d6a994b3e0fd402c5b33c97b4c00f2b65c393a29bb012c4a118a382e7b3187de7c6c97598576c65f99799").into()),
                    ethereum_beacon_client::PublicKey(hex!("cfc2d1985cac5a70574824953a34b2d553ef9dafc418b1cc0c925d21a72c4a8c779bb432d9622a0ce81161c32f295f2c").into()),
                    ethereum_beacon_client::PublicKey(hex!("862998ca5b4ceef04dc506cac25f9713308d61209cf9159fbd86c7f72c568d6dc30e4a41271f6098ad6468138c8df230").into()),
                    ethereum_beacon_client::PublicKey(hex!("0ea0ed2df34520c90e2c77b4280dcd2f8146d30d90abda608cdacf72386e65e72d3dd01c8aae38915cac31efe1c5f8ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("0903d2a58e3b23a08059bbdfd0e7921efb1b9f6ced498ddbe1ee4395ffdc0f76d5d62c608faac91d1af5557f61622cbe").into()),
                    ethereum_beacon_client::PublicKey(hex!("9cecb4609044d57b1a317115b51d903f924f7c719d7f3652c74df2db479a87d480d42e13dd413441d9ceda78f7a2bcb8").into()),
                    ethereum_beacon_client::PublicKey(hex!("00d1b9c442153143dd39352584f1dbb650b227830f16e17689252b30a1562c4b444d6ee2477ba869071b7b0da5c59c1e").into()),
                    ethereum_beacon_client::PublicKey(hex!("8e758e5a89f0cbcc2e20a1ba24645a06d5fd807b7b61905b5c94337a5c8ff1bce7c376052db3c0d5d9f5627b185e05d0").into()),
                    ethereum_beacon_client::PublicKey(hex!("ed2815d73a0e4eb8363dc0702791708d1414eace45cc72efec789b316f492615951c426ef3acd1b2b1b25d89a226659f").into()),
                    ethereum_beacon_client::PublicKey(hex!("5ec91e9b10f34a536bd19abbd3a8b5d8f449610bb29ead21a3d70214c4070c62495ab957e6562cc0b74249b5845951d1").into()),
                    ethereum_beacon_client::PublicKey(hex!("11470a00ce1b3b826c6b09dccdba674c2170faf2ba2493b99db913170bd32c230fc286a1c6ba91d8983276a3ab055782").into()),
                    ethereum_beacon_client::PublicKey(hex!("5cd57023a718919f95c57884e5f11d710dd8f368e3704c857aeec8832630d2a5931a98bd16217333a33a7a1689c3d967").into()),
                    ethereum_beacon_client::PublicKey(hex!("1bbbb1db07d866a6327a9d6fdcebbec692579b24dc40ffb92cd1f126b0b236be20e7a38e68f95bbc98328ee2152b492f").into()),
                    ethereum_beacon_client::PublicKey(hex!("c2d44c95d616d7557386995ea9d41aa85f2b5ecd59b7be030dedabcbcee59518beda22fc880e1b22d9490eab6dcb9104").into()),
                    ethereum_beacon_client::PublicKey(hex!("6674895bb092d1e3cc3417ea2883b6a183b369a94993c191b557004544cf10515a08078e88a3d50a40138c4a085d44de").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ed96cc214daffb4791b36556ffcfd86c530b897545a3115fe1e96b9a9b245a8ce13d1f79fd564676e412ca585e7a12f").into()),
                    ethereum_beacon_client::PublicKey(hex!("89c8e60701ea5ceb56c21ed2bb214cf11f5228809bd87d90cd498475b441538719ad9333a4518ff7de428da357cdef1e").into()),
                    ethereum_beacon_client::PublicKey(hex!("693e71cd15c563d5991f59b9272e56a0708d9c6153e5f2e4c360edd2216f0f34c84a0958657d9951f07adb59cede3872").into()),
                    ethereum_beacon_client::PublicKey(hex!("a321ac46c76d814041715969badb8387aae44605c5cb37603c1b5583cf86ddfcade68fcf527d7b311a671c34a74fb63f").into()),
                    ethereum_beacon_client::PublicKey(hex!("8b9dbe413c3daafc8af4d1a993693b2c2212300cfd0b845ad4fe2737987a94df2e674d09f3b696f86facd30ff51c3346").into()),
                    ethereum_beacon_client::PublicKey(hex!("919691e12cf32158e2a713d307a0c5c5cd12d7c599298507b5e862789ac06c53aa2887597726515e2b0c6b2bc1755931").into()),
                    ethereum_beacon_client::PublicKey(hex!("d7615e4bcff4c2ea320096157c9f6983bf7356e402d1ada1c533d9020a41b2c059c7b0998080c9cecae0429590126f4d").into()),
                    ethereum_beacon_client::PublicKey(hex!("46b75aa174b464929c3dad0612ffab3ac4a93d183acabf5c63b4028de74c54657bba088cacec2314f5faf010a0e0857b").into()),
                    ethereum_beacon_client::PublicKey(hex!("a6af7399d8d063427e7418186671c1865b92e341d1d30e2c2370d58bf7216d9ea45e8bc560247f4a12a3518a7d20bfa2").into()),
                    ethereum_beacon_client::PublicKey(hex!("aac38787d8d4967b681471588deb83e6076a75ba6f41acf9be9b1dc37a035612519b8ad3d40c3e9cb6705a50b26de504").into()),
                    ethereum_beacon_client::PublicKey(hex!("ac422b79df1dc95d28ae4e672fbcf3f3899e1154cfa49b557bf6bfdbdeceaa3035dc845e70fe13d97e984d641d61e670").into()),
                    ethereum_beacon_client::PublicKey(hex!("5465c9ca4ec21d1175f9cf57be625b8b81bfb1e397be86b65f12342223aabf10e933c52d2ff62c0c3cff5ecae48703de").into()),
                    ethereum_beacon_client::PublicKey(hex!("0002c940e0259645dbd45f8dae6e890badb365aebe711718ec289521683072036407d97173b75a1240460f016bd5568d").into()),
                    ethereum_beacon_client::PublicKey(hex!("03de8d3749469cf060c6cdb97e62faf4419213cdf2e200d3573af61a19c7cf99c227972e5eb76b1f7c0def16678c8a37").into()),
                    ethereum_beacon_client::PublicKey(hex!("327ca1925d2e483c14afcccfa717d2fca8d2317d1b9e1deb19143b154e3dd5a3a95a8771c22eec1400bfa3ef74d03b67").into()),
                    ethereum_beacon_client::PublicKey(hex!("6d7cd4ca1e676132f4a36df58a857017da927a84ff9a3b581041a80e9c16d1d1bbff804e9da701d44616a3ffe066f8dd").into()),
                    ethereum_beacon_client::PublicKey(hex!("65bef5d86a3052820aa4805ca39233e335d5df1ada012fd302ddf8e2b23638de3ac804551abbe17fbe847d29fc3ee94c").into()),
                    ethereum_beacon_client::PublicKey(hex!("171576e109389210615c80b32ba7a943717880e597d6f54e8ae7c91ab057eeab331abf182469e0bec0dc30fbbc069d9f").into()),
                    ethereum_beacon_client::PublicKey(hex!("7e0d9cf753824cd4eef7989d352e14245ac976aa7da97432de070ef286d4471a2326736b2e6e74e05e4fb501636a623b").into()),
                    ethereum_beacon_client::PublicKey(hex!("5b4480902f197a8dde0a96fd411d49c461396e9e2e6b668541aebc0c57ef2adc015184cc5d8c9a5dc1e208fc7441d8fa").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5b05f65db381d5b78d6abc874a96f330819bb19fe8f9c6f984030bbac9a0dfa2b23e5cd1f547d5fbe492b668013e51d").into()),
                    ethereum_beacon_client::PublicKey(hex!("81478b4f9dbcfea2e7d9e58fa722eb4e2a28f11ef2b8971ac901cc3513fb2bb3b06cfd636a48056a238b3d2f2d22bf39").into()),
                    ethereum_beacon_client::PublicKey(hex!("a598e546cb075bedfa78c70740378270c40df45b743d3129ab92a1890805af4092292ff40cfad3a1438825f0f3069b20").into()),
                    ethereum_beacon_client::PublicKey(hex!("29010a78762071447b3f6d3b523f0f9c770e742bbc6f93cdff5bc9c9316961ee8c50ed46405d2f7b2d30547092afd9f0").into()),
                    ethereum_beacon_client::PublicKey(hex!("c5584c4416396a1a0fee5f61d72fe2f959adedf469f6d4dfa6636e8260606817b88bfe8b9f26bbe2102df26f633c9535").into()),
                    ethereum_beacon_client::PublicKey(hex!("7f1fd7878f3a4d9eed3430f9ff89024b3ce386f2e5b796f844c29a557774ae129f00ed0bb01a0fe8ec2b7e7a84fcb493").into()),
                    ethereum_beacon_client::PublicKey(hex!("446d473edee8b199ae9054d79dfec55c125b2450795234d9439af2ebaf2af4f56a05b09ae2d632df9aa8b0a0d180513b").into()),
                    ethereum_beacon_client::PublicKey(hex!("2827d2414a76ecbd48643eb5ee7d14f0db6ae4e90575227f9b2e6a6ffc5ab315d57337fd30c7f259568f1dcb12ad84f6").into()),
                    ethereum_beacon_client::PublicKey(hex!("d751a8ac677460557ba93ba408b43acbb88ba332fd901897303a4db4ed63659420f5997f8476f3be9fe7e20e7e94e35e").into()),
                    ethereum_beacon_client::PublicKey(hex!("b8b7d1b69ddfdd838e99855caf768f7a2c0839e028758684e5bae7fd794d39b0c01b18e58edcab5864bef1ffcf9bc0e7").into()),
                    ethereum_beacon_client::PublicKey(hex!("a975ba4683293cddf7df96d86d27df93e8d3af3d8d9358a72aaa520aeed28d600fd1629327d2149c2a6fd2008c86e783").into()),
                    ethereum_beacon_client::PublicKey(hex!("a9db57c59cd11ac63b77f775b5a534e6bbc49c36cf14c5eb50d8c515d9db609932ebd38992d206b758f5bdf79fe9e19b").into()),
                    ethereum_beacon_client::PublicKey(hex!("1854a1702e2d88bf2599923b37e3ae7a215eb328f4411c6a027af018a389c2c48aef4da5c7ee5ca607a2df6504b063e5").into()),
                    ethereum_beacon_client::PublicKey(hex!("bde2165260aeec319eeb21d6e1887526c80073fa4cd645601ab18d89a273ae1f2bdfd001adec1c6a207e89f6df4b52d5").into()),
                    ethereum_beacon_client::PublicKey(hex!("ea1c1ab4b439976dc0a83a21d5950f0b23f4cd3f98fc74ee87e563ded51de6814a9858d726b72913fcc17326f8df1280").into()),
                    ethereum_beacon_client::PublicKey(hex!("23f245c03e8068492208f3be3eb7aa9b692be64d5a2fbbe67b970033d30f57bce82767e45c69ab73dc8979f3f42fd875").into()),
                    ethereum_beacon_client::PublicKey(hex!("04affc5e3bfb32e9da7cc1c8313885e3bc445f6bb07f98a8e8bdb6d8805cd97f824e82ed632781d76af6adaae9e2e35c").into()),
                    ethereum_beacon_client::PublicKey(hex!("317972cc1bf6bd50564ea1d4063dcac55b1ed54c33afe12a867e7a7217f09e761d3794de130005662abfc90e0ae1d9f9").into()),
                    ethereum_beacon_client::PublicKey(hex!("8e19957a89d1bb94ce7f5c66e66d1a738a974fb4d8ae0b9446d3e7455ccf9d39013d04d3c0818eb7b28e3ff188b6c8b2").into()),
                    ethereum_beacon_client::PublicKey(hex!("a7e4692ef517d6a222235971d63f8a9865fd2bf64679886b5865c8d93a3e92471595e6fa41d5306190aa755678069cf9").into()),
                    ethereum_beacon_client::PublicKey(hex!("aca2e3bef92b1caccfc99ccb13a5369c168d9e614035bee8005f94df9debd42fefdefb63085580c418c62cc87c54c9c1").into()),
                    ethereum_beacon_client::PublicKey(hex!("f407b443364414d4687e9c262edd8968f8227453e21a2dab0df192a398142eec25355d557bbe7617f7d6820c34afbb4b").into()),
                    ethereum_beacon_client::PublicKey(hex!("06226636518893196f04d110ef44234da6c03d42969cedc3b028cd4e2a2823380e37492f9d4082c8a840960e8f851193").into()),
                    ethereum_beacon_client::PublicKey(hex!("dbb0eec956c4c76e9289df2c3e11a4f63b747093c9ac9ccc425be23cb61236c4782696267dceb77dfe15e67608c53518").into()),
                    ethereum_beacon_client::PublicKey(hex!("12a0893a2fb2755fb06ac21c6fc1e9aacc94853a572ef142439e3933561f4bef2a3a068ab908156bb08045193c81b482").into()),
                    ethereum_beacon_client::PublicKey(hex!("2f4754f6755804d9e183db0bd0559c7914fcd1621818347711d9781ad538c989fe0943962918d3f48fd1f0ba2943ef9c").into()),
                    ethereum_beacon_client::PublicKey(hex!("daf3c4a105e481fc6ed7d7958cd6664a322ffc422bdb79837a3a2217a1c8953e610012b85e8684129ee214a1ed22c450").into()),
                    ethereum_beacon_client::PublicKey(hex!("e17993d7a4890b01dd06c17c449121ab6464381159116495549ebe5691fc8f789dd9f451205b6469da4ee5cb08bbccdd").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c6d39813d221f76daec53a4c0d856a6d81634c7eb9fcd4995000e6924ce5dfddd9a2f8bb7fa03588509d6f26cfe4b87").into()),
                    ethereum_beacon_client::PublicKey(hex!("4dbfc314db89655c3abf449f02501328775fc927486c6165617468ff9706fe4da6515801508745e182bb01c3dcccff0c").into()),
                    ethereum_beacon_client::PublicKey(hex!("2e78991588e809efdda4449cb641bb5c454c3323bb58d2ee36180b95bd38b5eb463a2ecea1d416f92434ab04c9b9d7b9").into()),
                    ethereum_beacon_client::PublicKey(hex!("8a4110d6e92c555aac0723575ade90028ac3bf3dba1e3d883b87e2d435644af1bf313a13aac4cf0de87a2074396a14bb").into()),
                    ethereum_beacon_client::PublicKey(hex!("e0c4efa0dd03954d62d199ad7986369946f5639ef6ba4ad2488f1778130e80deb3edca8ea8a764d14b8e26c8acd92d58").into()),
                    ethereum_beacon_client::PublicKey(hex!("84fd88d078841abaddb870ad7b4af018f89fbc870861f07ded34fbff3dda200f08c0baf3735de6f217ec2f22fb21167c").into()),
                    ethereum_beacon_client::PublicKey(hex!("57e5fc66ff7e8a6979b50182f51b949d4f6546aa9971edb222db251d53e35575d72c475ac7a3f89287be169a167c975d").into()),
                    ethereum_beacon_client::PublicKey(hex!("a941e1bbdfe8075dd8dcdd897825b9ae0700d6406c30cdf948cadf8c4e8f965d268cc7f139e14662b07375d729ab50ae").into()),
                    ethereum_beacon_client::PublicKey(hex!("80eab08327a584e92c7b407ddfbf2f2089304fa645025b64d320ad805faf848acdb0e50a4a83619c7387632def5c2c7d").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e336fbad48495662425322650229205ea9a0027f2fddf322e20c0c96cad584ff0d10e715ebc92a0f16df3626f0f50ac").into()),
                    ethereum_beacon_client::PublicKey(hex!("2675aac1369cdf82eda6070a4c26978b6cbdc087e2708031e499c864bf74f9b160a39c3133a7a54cf5010fea77a4b9e9").into()),
                    ethereum_beacon_client::PublicKey(hex!("c8551cebdbb6bf0cde14db43cc93e33f7011bc7ab20d0ee83aac1f1576136ba201e6eabdf89dd7d9b7301bc5d3e27667").into()),
                    ethereum_beacon_client::PublicKey(hex!("3ae7a17e2b3ae4db193c76edacfd835642c10c212686c2aaff001d61c0b2be5c0a597b61e4f5c34d75035fd314d61cb9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9b9dd6dc2ef294459927e4051e7b601a6a615aa245d2234b85550c02fedf4a7d708a8d42d74b486b7618e56122eeda37").into()),
                    ethereum_beacon_client::PublicKey(hex!("44ec4ad5f5d593a1d87b29437502f81ba83844070ed7768da1fc2c0d01b71e0bf81896f41b91eb30895556f4a8c9ebd6").into()),
                    ethereum_beacon_client::PublicKey(hex!("fcb1e1d118f1642150d8886ca4b8dc9a22cebe090b9fb3ed2a083bbbe7c4a1f294ca5baa0e7290aa3a451240165e39ff").into()),
                    ethereum_beacon_client::PublicKey(hex!("cc0f69f26185206d0a2ba0a81884907e68734f75a2c405b09fa6321d7d3c57758f95a30c303f8fcd552016473d61edc3").into()),
                    ethereum_beacon_client::PublicKey(hex!("ffb1ec77a144c1763d674c3f0ee3a68ce07a06ba800c5847cec05e2116eadb76ad04f878aca1958bb2caee4a83c4a89d").into()),
                    ethereum_beacon_client::PublicKey(hex!("2c1bdcf7927b6c8ef8102500acf5dd5a4311e2aeac743e18b6a1b9bb40aef3355e0186b1070a08011b4f7990c10c1eac").into()),
                    ethereum_beacon_client::PublicKey(hex!("2ec2e6ab03f661b05ff18fb1581dd54f06cbd51402da28d8c0af5e014e29bcb7931cb1e11119d169219fb23f8fded28a").into()),
                    ethereum_beacon_client::PublicKey(hex!("bb77ccf790d6a01fc15ec8cc05c556f205dd9c876c5f82c70c25cee59e4aea3d9f7796a46bb9427064d5ffee880b492c").into()),
                    ethereum_beacon_client::PublicKey(hex!("7c9df8d5b6a34a9ced1cd09c2bf4f70599f60c335a0c6771c5f04ec232cc08efd83f0944d3589956f60d252d845feb05").into()),
                    ethereum_beacon_client::PublicKey(hex!("3ab8e23133e45863418fe65445330a3b6c3011df8bac6cf93b5926b086b9dcd21049e015c7a8603e5e6f6cb1c7d82183").into()),
                    ethereum_beacon_client::PublicKey(hex!("640a25a74ac3a4b04a5b07677cd2ad37632e9b388ad7a33527034e88fba9d9fec741ededd0a0b24eeb6a14e26b79eb2b").into()),
                    ethereum_beacon_client::PublicKey(hex!("c019daa772c4b1e662fba4c7b8d747d063560cb8dcf096e8e76bd53c3313f1d798c8e6521ed3a9521b76d29589317c8f").into()),
                    ethereum_beacon_client::PublicKey(hex!("d3a15585a4cf0f70a4efe499c743f9a65fdae640a7aa9f544fe64ae0f063d1ddbb8ddbb00190393cd90d3c6888c0208e").into()),
                    ethereum_beacon_client::PublicKey(hex!("5e479a68d186de0b5f09520a57cd307b06486c58c62df60f4e0b59e7b6f37363a72525ec413748c08678d332454abf39").into()),
                    ethereum_beacon_client::PublicKey(hex!("587fdc44cd4e64605683aa5310cc25f5de61ba55e6092e0412073a57120b434362fbf166ca2899b005eadd041fb3c671").into()),
                    ethereum_beacon_client::PublicKey(hex!("79a8839537b861061451227ecf69ec2107ddb6639b58ed21e6c4a662f902b051a1c85c6560803be39c380107a377f9a0").into()),
                    ethereum_beacon_client::PublicKey(hex!("b76886ba071f3d36b68d859c92bc51d4e5c911996a85d9743e92a195fb8c431fb74acda134bcd0f0d26b919a76b4b55d").into()),
                    ethereum_beacon_client::PublicKey(hex!("20d812559299d8a04a6a94bf925fae22c76da82814e41d0c993f8bf33cc3ebf806f6cd8e0661ba17a94e8dbc62bb0c9d").into()),
                    ethereum_beacon_client::PublicKey(hex!("c5f0c2bfe5146258675a244f09181e13b0433e363d7a9140965607f73dab103ba7326241f488a4d59684c02865d55635").into()),
                    ethereum_beacon_client::PublicKey(hex!("c3d5d83f97faa9478167ec0a1ebe77c7a63ed6143d1a72dd2068499660943811b218941c58d741da20cffecac5c55990").into()),
                    ethereum_beacon_client::PublicKey(hex!("0b14f01c2fbd576354c92b5a0df3ce10b145929004d48971cd0e756c5f56b04aeeb34516f3a144d03f002d0967ef7cfe").into()),
                    ethereum_beacon_client::PublicKey(hex!("b260fb83e43dbf1a33d6cb248a915d68f34643f5a8314679def67fea3b177111a946b930e357c5bd9f2a2387aad080b7").into()),
                    ethereum_beacon_client::PublicKey(hex!("39312b58753e00d10569bf6ef24381f6617ee240ba544b1818bf872b6e879a54a2812af995faf863d87c291f74971c13").into()),
                    ethereum_beacon_client::PublicKey(hex!("b0cc618f13bdc63f993bfd9c481f6cb7811cf95f5f0245f79005fbcbbb8fc778a8c987434f14cbb93a45abf4b8c947d8").into()),
                    ethereum_beacon_client::PublicKey(hex!("2cc5db08ba74d8a775034fc0bd5c406404fd4d7676d44943e63c96bacbc39962f6d3e3500b1c644f77e712228fdea2cf").into()),
                    ethereum_beacon_client::PublicKey(hex!("49eb863bd11a2c4724a777b9d460eda0fda2c4becc672a803c48de9f96989568cb364099ac564dbeb3e92afa67f60395").into()),
                    ethereum_beacon_client::PublicKey(hex!("999f338eca9bd5f056804756a7d0d6fe583aa7f7dd649bf1f8f67968099cbd0b18940ec5067a7ab25e01e23051f2087c").into()),
                    ethereum_beacon_client::PublicKey(hex!("a01a49f1bf7fade0df36ae023fcd9cab0c1f4f03b5e706f1f56f6fd8039ec23de612c1dd733f9a4389d4eb36d56e07fc").into()),
                    ethereum_beacon_client::PublicKey(hex!("e336096c0ed659fcd9f1cc977ce102476e2dbc86d6fb5e02acc8917a54e5b5d622255df54b120c3346b7e5ba16520929").into()),
                    ethereum_beacon_client::PublicKey(hex!("91a23149c6166ccce189870bbe0d9bc8127794f7ff5457b81684a2b3d188f39dbdc4b93b72e9523a1900ba6f39a85ea3").into()),
                    ethereum_beacon_client::PublicKey(hex!("5fa3362b8142ac359635289f2fcc49ee26ea5e4713b09a627e092fb6b959dc47a73c9a46659ae3d262983b31b1fb8d93").into()),
                    ethereum_beacon_client::PublicKey(hex!("b8b4ada2cf1781b82769de6345f553fe4efa941a6812eccbb97f1569b4c850c3c0389965f9b3c76b0ca10236aaf8d3a4").into()),
                    ethereum_beacon_client::PublicKey(hex!("c948ada289c233b3424fce65c7b2080b8a8784523d6f2b96f58fbc378c2a02020ca52e86a4cad6572c54bf5c7e31c51a").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ac7ccc2588fe8659ce1b151aa71e412147c034d7cc6131d1ad227541a42d5bcb287a56ba541d4a56b0d6bd62822c1de").into()),
                    ethereum_beacon_client::PublicKey(hex!("00996c99f81b91725e1f69fe003a1740fc6f8f86e3669c6269a79f73856659111c67a9d6c909feccd1ed13031ef67f7b").into()),
                    ethereum_beacon_client::PublicKey(hex!("f565da15259bce6a355e272514500a64f769bbe064e039a47684905a4e2e42b3c568a1430a576bae5a896cdbf14998b2").into()),
                    ethereum_beacon_client::PublicKey(hex!("dc5bdfda4dc4267ad8270129bc97a7d5ccaa8bbbb50088e1e834c9b5bd3fd77bfb87ed2e1e4b8046f8dd969ea48b5a07").into()),
                    ethereum_beacon_client::PublicKey(hex!("227623b84b9ef970d38f0a6db223d994e9732b43a9db5df589b3580e063b5b6ebe39eea0fde1b49de24607bb972ec7a9").into()),
                    ethereum_beacon_client::PublicKey(hex!("79722813e02f532e1ce955472b3b92a0c65170041c66db8810644088ac1f9d7d5ec9d486892e4a2f8ffd585c39372e28").into()),
                    ethereum_beacon_client::PublicKey(hex!("d2db351f4afaa5d229fe2e51186f02930c2bf1714c7cc22fad5014f06d340fcf323526583faaec4b7b44c12446f53fbd").into()),
                    ethereum_beacon_client::PublicKey(hex!("c5c89944408497894df0906be5b3672400018951be3b3a96a891b26dd8a6c6623f70ca5d07d1881e168188ce7b0cddda").into()),
                    ethereum_beacon_client::PublicKey(hex!("1ca436bf1d6c5c448b641a280d8710c80b7d54243a1818e6e20bfc840ec5e1d6132f0e8ba9b9a9dec71e44a4547c5ab7").into()),
                    ethereum_beacon_client::PublicKey(hex!("36bcb436a5b15d3f6fda0fd15a428e408e4e8b5dd14524a495e671f3ac601751a08398fed5da5af8a466aef08c5b4f54").into()),
                    ethereum_beacon_client::PublicKey(hex!("6bff71d59b403ea90033ce90656b620bb9fc24a0016656c2dd7bfb3b0313647a0a938b62f723a9950d1deba7f6fb7fe8").into()),
                    ethereum_beacon_client::PublicKey(hex!("f7ae2b7067ab87306721ea27eb772fadf6171f44578e00c750a328a89cf6307d26cdf130d4f3fd775bde1945f90bc88b").into()),
                    ethereum_beacon_client::PublicKey(hex!("be23ea95fe9f4c3e0ce3f100bb987ce1217ac3a13ba7f36e7548a4b39295a239810070e373a3b192a104c4098b3fd1bf").into()),
                    ethereum_beacon_client::PublicKey(hex!("e4a3b7ae6102ddfb885af9f00b1ea9635800585ec63859d950488d61541e5aa21156e9bd59b8eccb5ed199e0137bdabf").into()),
                    ethereum_beacon_client::PublicKey(hex!("e4315f35a3c07600f3fcd7189b7b13bdf0acf49a62ed3feb7c7ee5d144a815d914671fbf6ca7e4774e81e9f23df58594").into()),
                    ethereum_beacon_client::PublicKey(hex!("fac458b869238514af57043e5ffacf21f41c65c8ebad331891d0d9ea0e5bf7cf812a1c6a8464040268c1c3649bdf9573").into()),
                    ethereum_beacon_client::PublicKey(hex!("b79731448d23697f2ad9e46d45d5294c579552050249c30986fec03aee25418e43bd21bb7e3c60b1cfb7e516c8457b25").into()),
                    ethereum_beacon_client::PublicKey(hex!("0df17cb29cc7d29836b52f814df0a8925d40bc8600eb508483ac46625496f78dd0168ae443814f568a86fdc4df31345a").into()),
                    ethereum_beacon_client::PublicKey(hex!("59be6134134684c549242957ebdb7b5d70feb9deaacbb9a7025f7c42332679badbd3a5f6dedea499bd1a70c26673aad1").into()),
                    ethereum_beacon_client::PublicKey(hex!("680e7397fec66aed567d8c8be353ee84af202fcca51722baf7f0efef64efec475a029f3c9d512d68d50814aaec677c76").into()),
                    ethereum_beacon_client::PublicKey(hex!("2dc1781e3f3ba5da4111c00fe219d57107edc8fda24a17425109f0f5357ddfc884a1511a811a8afda823a5d0d24f618c").into()),
                    ethereum_beacon_client::PublicKey(hex!("1d87d8bf86a62958992b2563206c2ba9ec840820dabba6dd2207ec37c8e2bf9d8ee3a32b41e7547c484cf9e142892eba").into()),
                    ethereum_beacon_client::PublicKey(hex!("f161e244d6c811eb53b3c4fbb3594b58b67b2d88da032e948b747c7b3bff3d6a0a8d8c645288e4d977882d974ea0e6fd").into()),
                    ethereum_beacon_client::PublicKey(hex!("8879dbdcb46f7cf04858250390dcb8f2c6d50f467a50ebda76b13e3065a441545a806ffe1ae8b1f558a1c614d3838958").into()),
                    ethereum_beacon_client::PublicKey(hex!("861e73a81cbddf6952f0f78306d0c08eeb9ca2e4bf0ae1c15c8f85b09a25232054ae4a3947df4418811a4adb202b9948").into()),
                    ethereum_beacon_client::PublicKey(hex!("2c636d623b2c1f429cfe607bca34d030b672f4dd5f179261145820566827ee73c0d8bfc0a4b1f859f9705f41a331d192").into()),
                    ethereum_beacon_client::PublicKey(hex!("1f8a1df647ac82a65d5e84e2370aea8e136896735ad7a79c5fdc02cbb7f395a0d1bf2e12b05e6f3602fc2eedba9aeae5").into()),
                    ethereum_beacon_client::PublicKey(hex!("fb04c8d5b7d81c2cf8b97b0b44c4b6c5f9e11e7ebeafd7e93ce1069db6383f3b4a2b545bb45ee515fe329ccdc546c0b1").into()),
                    ethereum_beacon_client::PublicKey(hex!("8b42f9688b1b86f723c971ddf1049aebbfa0e8d07ea2d2c336228e75bfd08b68196e85f407f2b56ab99099bf52ecb071").into()),
                    ethereum_beacon_client::PublicKey(hex!("8879a7c6dea69e0f219b871a532631b9ab1232afdf39018d74211f1992683e69dad55c6dcea230116c2ed6543a9f490f").into()),
                    ethereum_beacon_client::PublicKey(hex!("8cbcb5718b3402dd26d178babae08d6ef0c34c023e6ddc1efb06d38bb010c114b72d762fbb3791f77565668dd77b9b1b").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab284c824a8ecb4035d8dca505b7b1ecbf9e1c773b27739b208a0edc17857e83b6a338e8ffdc855fce5b5bb3f02c5c0d").into()),
                    ethereum_beacon_client::PublicKey(hex!("6b4b1f780e1ef845065da669986b0bf4e84391989344007d0b5de73d90e8a9477ae475276c292dff360fc72aa7d63dd9").into()),
                    ethereum_beacon_client::PublicKey(hex!("b28a37f6816e165cf446f2f3f16efdd9aafb7ef2c8dc485ed75c67d1d1fb18668ad3ff60929320ca7a105caa0274c902").into()),
                    ethereum_beacon_client::PublicKey(hex!("d647e816ffe87d232539a781b42e22a9cf526d741d4657e5cf9a140e28c386c5053346311103b355a4776b833b20e2d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9b5707472b103f1e90760dcf1cf9e3f8a48eb3f8f0f6456d210e71dc8a2110922075dd3368b8c641513d3befe62aed2f").into()),
                    ethereum_beacon_client::PublicKey(hex!("e5f013fb8c9bbbf73fdc36d5ff49433bca7cfdbd3510de610ef966b90ea8a14beec7c37298b26fc79b762cb0056e39a8").into()),
                    ethereum_beacon_client::PublicKey(hex!("22f9523123f339946c4f3d30b854e214fa1778bb0c81ce64597d0666c31c4a694fa59ad916c52b470b148c798db9de5d").into()),
                    ethereum_beacon_client::PublicKey(hex!("3351cf4cbcda72d40d43a433e837eefa8f7a291a2e21d8f787c04cbc2921956930a4cf7f653b47e6025198d1d7f84e86").into()),
                    ethereum_beacon_client::PublicKey(hex!("9264b32abbdac89b424ddd852ce1f5f81136b72247ec9a81e78417bdfd575dc5242b56d42d029480e96926657e9883ec").into()),
                    ethereum_beacon_client::PublicKey(hex!("e1b34041c02cb47e9b98977f87479973b5f0c126a8e50fa3da15ee6c5ea0fbc3ea96ba3c3ea41352895bda6bc43be154").into()),
                    ethereum_beacon_client::PublicKey(hex!("fa04e07fde6350b0783872277aa7ddf0dcadb71c417d6f5ad5fd5f996e4b8c1529a4c1b9549bf1b9470d812bed2f0219").into()),
                    ethereum_beacon_client::PublicKey(hex!("53226ebe9468f67f6cf5029718436f1d0927b69b7a4ce65461434464623ac8b8cc435c79bb893aa1eb9b8e61282b7b81").into()),
                    ethereum_beacon_client::PublicKey(hex!("06cef8a94117cb83cdfa690e17c6469f5dfa2ba21d15bfd45a85de5e492af0ef092c6a92a156b458f2530786dcc59f0a").into()),
                    ethereum_beacon_client::PublicKey(hex!("8bd32db5735956a9ccf4d8735943fb2a23034c879cd95f3f80c4de0b9cc20078c2c13630bc28280e080530b797bd96c6").into()),
                    ethereum_beacon_client::PublicKey(hex!("df497ee408435c72a92be5f7e4a19d72eb5162de18d5b7dcd38716be0d18750b55165b2682bb917c7095978826973f6a").into()),
                    ethereum_beacon_client::PublicKey(hex!("c3c02a5675a6e54e2a0d98911fca10818c1e8ab62d504518b4f54a1eaa496f2a412911d31b14e887806ab9cf92ad3180").into()),
                    ethereum_beacon_client::PublicKey(hex!("4c9f52876e3691d0ed44abfccb13bd3958ed6132dc0b6c3388e027005831575ec1c274800d30f8b45e11bb5b6996651a").into()),
                    ethereum_beacon_client::PublicKey(hex!("b3dbdce3b154993a517a460cfbb4e480b0b793d4da76869e6cfa042379085cb38ee99e646f75507bdf3f13322a6d1f97").into()),
                    ethereum_beacon_client::PublicKey(hex!("cb7f417bb8eb482be13bb476cbb72d12a3428005f14569e8e391ec14b0488310c6a7ea00d0127b9ba0423c951fd6f0cb").into()),
                    ethereum_beacon_client::PublicKey(hex!("2ba292f9d3821a2a1bd88266f0f204c202dc71ed36fe27d465f3174b76d82fb19460237d1a26e7f17e1f9235374a7f60").into()),
                    ethereum_beacon_client::PublicKey(hex!("c623a9e36119a1ab0e01740286a6081d9fb817c72ccca4f9d2df59a08e3a83043491f44abe09ada15720a1964ed20b59").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ef3a922f83c584f56140985bbed54c93db4ac627eb73a424edbb50485efd8d280e5a97a01ce43ccd6758ebe09181317").into()),
                    ethereum_beacon_client::PublicKey(hex!("dac5d8f1639830cc2b1b17506c0078aca75139d784b499ea9cbb4223a6fa6f931b04fb30391d3855678409a60772afc6").into()),
                    ethereum_beacon_client::PublicKey(hex!("339237bccca58405048e28a90228e07cc03392de6caec556ab825db2e5627f372a84ab0e36e21e89aa25e2df9deaa004").into()),
                    ethereum_beacon_client::PublicKey(hex!("b1ca31182f1dc721417fe6dba5c9196bccddb0c2d5dd8aba9878c8c2374c1fd69a60c8e03fe8e166940e7a8098cbb35f").into()),
                    ethereum_beacon_client::PublicKey(hex!("c2f884459430cb3723f2d90e9de32d3ef8f01af0a885695c5dd4d0513010e4b5d4b9db12d6dc350555f29179c93ab4f6").into()),
                    ethereum_beacon_client::PublicKey(hex!("4659ad1a5e30c78808437382e98c164affea4fd333a6e989fbf6c06c6124d4e986f43e53624f886c1bbb55222e2767de").into()),
                    ethereum_beacon_client::PublicKey(hex!("da532030048916d9004d965be1fe05f6b400bcb5e8d1ec601f32ffb3b0a1ba4dc264552683e6543616423de2e5f0bd27").into()),
                    ethereum_beacon_client::PublicKey(hex!("773b6e459689d297f3d7a9276eb38bf0d52dbcd6cd191844f1dc77cf4203e562daa08613b10853363046dd6e9fc5816b").into()),
                    ethereum_beacon_client::PublicKey(hex!("f67d682f27e19b5826865cc53b3902087bb4f7f463791717f9fbf7922d366fd6092816426855dcdabf63c50ca128adcd").into()),
                    ethereum_beacon_client::PublicKey(hex!("b7a1683fb46c615283239bdecffa64a424bf50386ae05f21a99b2dbc865c1040f23c7f9b822c07935aa117abaf3b5b0e").into()),
                    ethereum_beacon_client::PublicKey(hex!("b3292c1567e69cfda914af4a88b9b3866da19b75b73cf26e9dc4913605db825caf32cfc682a5a2cf44174f852ae18f53").into()),
                    ethereum_beacon_client::PublicKey(hex!("e86de28f261e5d99d1018787536891768cb3a637c058f3a907d77bacf585a868472885a9d9f040f604d91ca0df38ae07").into()),
                    ethereum_beacon_client::PublicKey(hex!("964c67c3f0ba0181be168f06a3987092752913e3dc74bf6cf959b3a4f2d495f9b9fd1719db92365c93a287d59c76d0cc").into()),
                    ethereum_beacon_client::PublicKey(hex!("ecc3011b11c8531a3210e521905ea5dae9fa513dd065f58bdf8c3cd2a0a188c199ff2f9a28eb4ec9a552270b9d9d1751").into()),
                    ethereum_beacon_client::PublicKey(hex!("8217dc346983f0b346c75fb52ed58555b9ff50b1def6241f98577602a4b1b920e7bcb648f24d0bfb0d09342c07469905").into()),
                    ethereum_beacon_client::PublicKey(hex!("48472011869e7f1cccbab985aa626feb791a046d0909e5bc8a7f835c1f2784d0b9ffb94814befcf6bd70d319a5afab07").into()),
                    ethereum_beacon_client::PublicKey(hex!("05524c57200fda8ae5584c903d99cb0f4e8814f432320a47cd6ef6e96c593c35c674c064c7458cde498111a3e323397e").into()),
                    ethereum_beacon_client::PublicKey(hex!("48b61fd688a92d00f31e84fd61b1d17e912ebaddd9a6c2ccd828b29513161205748e3e8b0ab73f612497d59d6ad0221e").into()),
                    ethereum_beacon_client::PublicKey(hex!("8f73077c785aed209e6243846f227576efeddde353549df21b9a04ce427a5c8d0620925bb483d4e894c8410c882159ea").into()),
                    ethereum_beacon_client::PublicKey(hex!("ab4c22f55fe68b5445569ebbfb81ee9230488437f1881b4cee7cce94dca1eac2918a12923daef44b60cd36284584d48c").into()),
                    ethereum_beacon_client::PublicKey(hex!("9be505314d4b077b96420c2db86fc897d91d1ead77a1c342127be77e692c789d7cc2b5f5a8c8efebe8a02d9cd211a131").into()),
                    ethereum_beacon_client::PublicKey(hex!("1e118d95bbd4a352b244e25aa8d583b9b3d27bebc1567fe913c044fb16c84c6445068cd5a4a3de68598c0c94d3f09769").into()),
                    ethereum_beacon_client::PublicKey(hex!("ce56a0b1299d8790a6c0c42acd868d0fa35741331542e187c17f42155bd25988f5eaf31c6eb8879dac1dea235f3bd51b").into()),
                    ethereum_beacon_client::PublicKey(hex!("79448cc53228295b30d5fff2a474276ba7a544d5726b5f35158b4ce8734f011eb71a2a5ef96cbf7ea6fcd5ff5c4aa7fa").into()),
                    ethereum_beacon_client::PublicKey(hex!("07a4a1e44de9bc9df1230fa27442e70745349ee65929915a012c894f342aa03b2d7f246ed0c6073654a8ef8e7375933c").into()),
                    ethereum_beacon_client::PublicKey(hex!("42b3ad402a73b649665692b4a4436c321bf7de4f620fc4d4c36f78ae84191ccc320411752bc5b9b6ddc6d9a0736bd431").into()),
                    ethereum_beacon_client::PublicKey(hex!("d6ceb58d704a23ccfa67094a7dc489d8462ea2e804da64d13cea5d3a15977d1d7887c0b6fda094da3068e0789020cfb9").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e991c3af0a037e5c72c9ee8573cf73a2b3070cfc14677e8276b469b0a5e241883676b612a097e8e35bc85686fef0cc9").into()),
                    ethereum_beacon_client::PublicKey(hex!("09b8965928378c34d661830080e2248a8f4d3ebf8d276d0cf46d463572994f8e1a8838af077b8aa3bb36c01ef1572040").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a969a3725ab6159d65dacfe0deeda2c3f68573ede5edd41169c5cc40ce2db7c8be23ccee5c3b84a7017e828981c0fb9").into()),
                    ethereum_beacon_client::PublicKey(hex!("9fc543c1ffbb92090af9fb95ad4fffe8c1e42f2361b88541def0a437d4a1852a52322cbc849470f511e85dd2b1d0239a").into()),
                    ethereum_beacon_client::PublicKey(hex!("6be2fd1a32d4e5b24551f4bbd4bc58036d287ecf06ed08f83e2e5ff8e1060bf16335db0aee63a9f3b91e0b5d73e0754d").into()),
                    ethereum_beacon_client::PublicKey(hex!("43dfab2e051a407901c72067b8908960ac7dc26267e7d8139e1c2f14765ffb9d423473c5908c02c6b916ddd324b1ff3c").into()),
                    ethereum_beacon_client::PublicKey(hex!("efc620ef4ad1c17992e385bb309c72384df98e5b554b234d81c4163585c2fb8f950967ff7e38bdb871c6837a6275d81a").into()),
                    ethereum_beacon_client::PublicKey(hex!("2db3b9183484bc2dcf2a95006966df60f1489221bb9bdeddcd2bd4a6c47eec5c8c32a0e84d6aa8063cf989ff75134d26").into()),
                    ethereum_beacon_client::PublicKey(hex!("c19f9a4ccfe34103c5d844dbdd5a855770f7b51ed07c326019c2378f3a64aec64523d6f6fd4035993f520cd50ed327b0").into()),
                    ethereum_beacon_client::PublicKey(hex!("e3d13221be67584ec7a210f48269c2225b016a7e76c5deb0bc889390cce68d9f9c9cb885236d726b3908a71bae4d949e").into()),
                    ethereum_beacon_client::PublicKey(hex!("6f06816cae1258cc8afa8d568624adb7f4d4c4a092046b1419bee96f7a89be275a86550ff7e28d08107ebf90eeb1a3cd").into()),
                    ethereum_beacon_client::PublicKey(hex!("b21bd20249cdda7ec9c9091ab4a9c82da6b29bebc9b93863ac9be37ff68e31bf096d6102a62a673b4f3993265a4e7269").into()),
                    ethereum_beacon_client::PublicKey(hex!("4bcfd90138307d30bcd433a0c4dd93a3b9149ddb3b18f37b83dbf8afe4874b41e72d71f9fa5bbc110c986643c09d7b8f").into()),
                    ethereum_beacon_client::PublicKey(hex!("32d84a7fc81c4cad9cf0e24025542484add41c2956893a2dc559475fc49828e1411f8c645efe2ced0dbf33eee577ffa0").into()),
                    ethereum_beacon_client::PublicKey(hex!("53016f6f7b251a963983ea7faa6acf1716305a73081556ed688a10de520680175bcc0bff7db6f970c8af73993865782b").into()),
                    ethereum_beacon_client::PublicKey(hex!("ee17f2750bd819d575413211932d135b40ad2238449828db386e95e5f00d175e7e5d45ee9b17043a3345c0b35a1eaee6").into()),
                    ethereum_beacon_client::PublicKey(hex!("2bfea1200d1e1c316d0f4ad3a9a812bc3f512f05234c0c255ed769d1c8e6b39eb37f187bd6ee682067b325fefb73a6d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("328a22d8f820948445c2f675bbd08977e849251c6a88ca09fdb973d7bac30f548b48206a1f0878ed51a1e1842c27f291").into()),
                    ethereum_beacon_client::PublicKey(hex!("5de4775658c19b18e6e57c87b80e78c7a5d98f800ac286a549cdcda95b2cdbe129162ceba3ff5fa30c6ef51c08c72c70").into()),
                    ethereum_beacon_client::PublicKey(hex!("0025f9d4c482ea697b48487b2f9598a56baaadb20a8132305785c1fe74c4e08736616025148303c732da5b6f72403687").into()),
                    ethereum_beacon_client::PublicKey(hex!("5864cb038a65dd8d60ca8d5381486e030bfa6b4e0f1576165964fa0b91b6fb15ce2519f6f3b60a46d8af658f0f70b115").into()),
                    ethereum_beacon_client::PublicKey(hex!("e0e24e558af53a16356c8a64f36323499b86ea074b64998a34093c888d33534a1b950345715b5e6ea4c0c8886546f021").into()),
                    ethereum_beacon_client::PublicKey(hex!("b1f58c8fb7bde85054e647f4d37ab48907903a575cfdd7ebf18f2ba031d8a0d55abf88700b56f1dd80f7b43985e2ea83").into()),
                    ethereum_beacon_client::PublicKey(hex!("8ef3e2050988e8f9115377b3870541b6cdafa0c41d5cf60243fc2aa0e5bcc3e5a26e56afa204bd46fcd448bd32557d71").into()),
                    ethereum_beacon_client::PublicKey(hex!("0097a885e5134288487ac7c74e816ec7134f29991d3ad82c1fe8a2aa37a0d0657375a0d9703b944a783f696da26b507e").into()),
                    ethereum_beacon_client::PublicKey(hex!("b57073f55798da723cbae804377e6ef3f3d652703f31837f38ce0cbed8ad6986a878f9e8b02ff7664740f5d33133a408").into()),
                    ethereum_beacon_client::PublicKey(hex!("d090df80de6d303e7b6a12463826fe6f189fd73b56cbe233a677fec9bae042a8eaebb7b543b6ecfe0f894c64340ce73d").into()),
                    ethereum_beacon_client::PublicKey(hex!("fbd2771eb7afe646e753e1143fc71c6242d08d15f3eb4661f64a57432b244347d0224e528c95acb45c0b115a6bc32eef").into()),
                    ethereum_beacon_client::PublicKey(hex!("90c1ccec5b07e1dfa7b11ed197ca5750e912bdbc08cd154285988657c17d3a9dd2d6a0e2a0637c4111a64b664c0dc2d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5de3fdccc7d764cbaac240297160ab8c03006d9e1c58ba5e238723df8938ad41513493ac70bab651e17fde8761faa02").into()),
                    ethereum_beacon_client::PublicKey(hex!("af0eb8983937b20e9a58efcbefcc38edd2e06ba855f936c919ea8b87840e42b8c52a7c27c989eadaa05fa76bf660d32f").into()),
                    ethereum_beacon_client::PublicKey(hex!("b9e9c1b2423e75a9c1b94820e5a4af02b80976b6c8b137fe84c43685f6a3221fbe156049e79a4956297decf4a11a5fe3").into()),
                    ethereum_beacon_client::PublicKey(hex!("909824631d1d54108084f51262b6de477db008882286ff0fe05311678a1a51eb266c012fb2f98ae77c5717b93a8d52c7").into()),
                    ethereum_beacon_client::PublicKey(hex!("5c4bcabd58b280f525c603f74b36c7bc27055b17e9e9db6a2c3fadd9a2e81d7836cf2928a92d0d92828828a43dd9f207").into()),
                    ethereum_beacon_client::PublicKey(hex!("6ec32e0bee65b47405cdc2e18920b3e44375411e8bd36f0e8176bbf68caa48eaa7ae70ac440f12b2ea58ae1be864d36e").into()),
                    ethereum_beacon_client::PublicKey(hex!("0f8b71ae2cd16f79792b6f9d84da3472a879784bfb9af668ee94f04b135a3cf5ba63ddf93965cffac77dc47f6db95978").into()),
                    ethereum_beacon_client::PublicKey(hex!("9c1aececbe3fdb59b247977f848885b15f7ef304d0f80e82255c0e0127ab69a0cf16d40969c2d9a865cd5bbdd0a3f545").into()),
                    ethereum_beacon_client::PublicKey(hex!("5e04d97939ac15c12d4f8fbefa99a92db45127e8de1d8e54ecdc3431431dffc7cdb9075dc6727fa84cd7f48a5818efdf").into()),
                    ethereum_beacon_client::PublicKey(hex!("2b919e7b0859fc003180bb10cb9394fe482121ad6c36ef9980b2e9eda3f4503558be73e92a86c9b808a7201741d146d7").into()),
                    ethereum_beacon_client::PublicKey(hex!("deac924f8c0d3bf2c48bddcb580e405d292369aa3099427f46b9ee542b9ae101ce992a3c0fae25d4af37322194db2383").into()),
                    ethereum_beacon_client::PublicKey(hex!("cc22f6509ca9f0670ffaa559fd93f9bebba3c779fbbf6f835d416f8784bae7c3eb1768b75b7e568f6f269207fe18719a").into()),
                    ethereum_beacon_client::PublicKey(hex!("b5a9edf1fa24a06c51e7e28e6b1b849492850873db39ea98ce6fe7ba2ceebbdeb11c4aac7678f22f5952d9f6f094eeb7").into()),
                    ethereum_beacon_client::PublicKey(hex!("216a07dc89189a198be0a2218bcc886f8fc23f58dcc4d805b5d54bab19c1a1f9903fe05a23b0e07f68c229360cd90aa5").into()),
                    ethereum_beacon_client::PublicKey(hex!("692aaaff1e314ed4aa0030f23f5ba4792fcae751f63aaa2821a8ef4980f87c04f5fd5f4fa4fa4760207b6bd685a75387").into()),
                    ethereum_beacon_client::PublicKey(hex!("f73631bc70eaf9b3791c3ace9bd2a98ba01f5d12b4956cd3ce54abc31326937abeda7cb46b7242ae1c11e59cf49a299c").into()),
                    ethereum_beacon_client::PublicKey(hex!("ecb748df4e998ffcfd528ad0a1182fd5b3b6ad2bd13b27f269f588abfa86dad4f7e3f47eaeff96a4cb6ff2394fb85673").into()),
                    ethereum_beacon_client::PublicKey(hex!("1e9db5aecb6e77f567ee6c1c69c87fbb4c05d3dadc84ff3fde2aded0e7719be3e944c79c574da604a5d6db4afdb8e83d").into()),
                    ethereum_beacon_client::PublicKey(hex!("7c8331dae5e6dea66381992ff095ef389d91fc567834b6ccabba82504b4a64fd224dd6bc64a21525bc72b54e89c65947").into()),
                    ethereum_beacon_client::PublicKey(hex!("00f3a46c1d5a7ddfc21542ae6bfa0d7d11b817e3b1cec928989eab479d188c0ace06422e0eef06ca4cb3b6bb2ae743a1").into()),
                    ethereum_beacon_client::PublicKey(hex!("e6d91059c8923dcd8c1ba7d204eb26129d8e1cb615560f3361fdb336a21a9b8d388d0c4ec5c8c1125c74b651b8a87447").into()),
                    ethereum_beacon_client::PublicKey(hex!("1b81e683c1c8951ef9f2d60f8d6981ff76cabd9f244b76fa95935767ae72aaa11439ca748d4ffa86c420e845c9c091d9").into()),
                    ethereum_beacon_client::PublicKey(hex!("a939e87acaf2364d0eb20c6e5fb34ca840ec59d24c3890faaebb4091003b95e8591a06d2e97040e08f62ae109041c30e").into()),
                    ethereum_beacon_client::PublicKey(hex!("0273315dab1ba0569d894d2d6ea1bfe337ad96c2ba698251cb6ce07fec84edce2d007bfca8484d97fcd495a984113576").into()),
                    ethereum_beacon_client::PublicKey(hex!("5b57c54165d85997ee71fddab0a6307c7c91853e2a7dff55f9fb9f819f34ba96bac7649d58c3fcec3089960e70c3f739").into()),
                    ethereum_beacon_client::PublicKey(hex!("1a9724785a156b5cbc04bd023250ebecf5eca79c867182ff74909e0a3aa36e561f7a7e5a9d9824f3b1f0de7ac7e3e66a").into()),
                    ethereum_beacon_client::PublicKey(hex!("bbc1ba4a1744686d05840bdc2f8598dc1836e303ddfe2de3bbc289dd65e316db0ac9825dd52b461d070be6037183af21").into()),
                    ethereum_beacon_client::PublicKey(hex!("a76a6e761dab25f53ba65a6339e658a99fee356513491e6a9c8bf5fd1adfdaa82a61c5c3eab29aaaf18d26d7a68cf7db").into()),
                    ethereum_beacon_client::PublicKey(hex!("e3059e67ab8e7a9008fa04b2783ae24cec8bc367f243f0829b997bbc6384e25c9826e6f7645e886473f601f7cc64e567").into()),
                    ethereum_beacon_client::PublicKey(hex!("3a663435112ca1274e865197c6602d9f5d02e8b7e289f6c1131c5d6b0810a5ddf88dd5c1d76eef1d4442fa4833f82b4a").into()),
                    ethereum_beacon_client::PublicKey(hex!("623fe7520964ed389f3225eb8b9576ce0dacc940de80c268698eebcc3c91b84abc6dcb8d349dd72cdc277a07556e797d").into()),
                    ethereum_beacon_client::PublicKey(hex!("78ca48e60d53c1b1847429d4cce2eb429009c58047764c07bf9fde84a5a3476f364f8dafe8bb167b2cccf2f416472a60").into()),
                    ethereum_beacon_client::PublicKey(hex!("47226fb6d8fae3bbf7bed1597bd84569a357bd53d949cdde3992c7f81643724ee9f72b62696c6396b4c637259e73a868").into()),
                    ethereum_beacon_client::PublicKey(hex!("0bb1b6692e799e18e77336b76c946464ebd1e558f894919cd997c086642e9af5f12c854ce6deb94ec527cb7e56a0829c").into()),
                    ethereum_beacon_client::PublicKey(hex!("8c5a4d571bc5749fb63c4c79134d0a39a15cf1dc6c5be170a516c523e1106453cfd45f04d0ec7445ee06bd72e7b2cb00").into()),
                    ethereum_beacon_client::PublicKey(hex!("27ad8315984488f302122db80849256283fbe2a36b292453b41718a5014668b57d47b0ea16c863fcf83675a8840bff96").into()),
                    ethereum_beacon_client::PublicKey(hex!("56209effafa71998e158bd7dc4a1150a2eb6a01f8c5ee7755ba0fc878db9a6bc6f69173f1a622500228581e0c4775023").into()),
                    ethereum_beacon_client::PublicKey(hex!("f7b891dc458730c26317850bdc4968c5cea62b81d558d52a6e822b357e7bc81807465fc60970ea675f5dcbab6392e3c6").into()),
                    ethereum_beacon_client::PublicKey(hex!("b34b3572d23f0dd027675e218364bad1e9bf3a3d9390e381658218afbc814efe6189f81e58b73e62c754d99c987b6ebb").into()),
                    ethereum_beacon_client::PublicKey(hex!("8faa50f5a25c0add8d9609af6c4e62a900016cd73d9767300e86e273dd5b893e1497a15c06efdb841ef03fe3f20bbe25").into()),
                    ethereum_beacon_client::PublicKey(hex!("dd02eac3e5f9279daed0f72b333571d199836098879a24afe7c1966baa788b27d7cfb3b9d84fceab4e484f895df3f28f").into()),
                    ethereum_beacon_client::PublicKey(hex!("50d78c9e44d1b5bcb0f0b6f65c7a8e1f915144b8112e8ed9375c0911f41398573f5acc99d765ba8f5a0d2f47434f4914").into()),
                    ethereum_beacon_client::PublicKey(hex!("3b338724435e729ec971de9e8e5b250e3d915cd7c2248c3007ed98d9245edacc0097f9a863e120b2d9dcb7fb747eb7cc").into()),
                    ethereum_beacon_client::PublicKey(hex!("6a3708c1f41d66f0e65e7eb17e826d4f1fcce407ed89742f14c355a35d02ae82c1c7958b4268613e6e8216ab0557fb69").into()),
                    ethereum_beacon_client::PublicKey(hex!("4e8317360c663966d6ac37784e2f0fa2f7a0dbdfe3676b92d693ba921eb9cb5c5fd4d6d659bcb5bbb9e7abca4d9c522d").into()),
                    ethereum_beacon_client::PublicKey(hex!("41a42ac00447280d00e2cbf959164915f1db6b8d50650c5b8a5cde7ab03b57a9202f55e5d26c4702a701c97eaa889e6f").into()),
                    ethereum_beacon_client::PublicKey(hex!("8d95af7e59b360612767991da1d059c6b03c845e748f137fe7c478d1dd9ab38e7eff333ba846a8042e4704570d611ce4").into()),
                    ethereum_beacon_client::PublicKey(hex!("f1a1e3f9220e944d36e7f31d5a7e8d0e74b5a5a86e284b5812e004abe6ba6f05764a1590465022c95d326d523ff712e2").into()),
                    ethereum_beacon_client::PublicKey(hex!("693ddb61295e04972b014d57f90ab4d0f4cabe17ddbcf1d2f2bbd9138a0415c3882274874ec3d58eb75c3ca589e7c01e").into()),
                    ethereum_beacon_client::PublicKey(hex!("3931ce237b2d97f62efff4139b6aa896ecae55cbbaa55362e4167da818c1781bd6c52880f770c62234bcd250811c1e8f").into()),
                    ethereum_beacon_client::PublicKey(hex!("c0f8be111b73b732a9742a3f6ddb0082b30b71c252ac9848279bdf22ee4c1a38a16c8e2493afc8d544c0da582e843d3e").into()),
                    ethereum_beacon_client::PublicKey(hex!("d58e670e2e65ecec6cfaee5cef86b02f3213d6c2ecc5e985a463212a437a658ce0e36eea6b5c558111e65a3aede73144").into()),
                    ethereum_beacon_client::PublicKey(hex!("4f8d870bf79765b1452408dab2e37f4fc0287606c2d5f6bfa8eb7ab0040e86116034692b3c54d11a99932f9cb9bc66e9").into()),
                    ethereum_beacon_client::PublicKey(hex!("925088675e26e9a11354152c1b22f3c1674b6c5f654ca4e77558ac862adfe1a43fe06fb99cd38b0337d1647de2cfdf79").into()),
                    ethereum_beacon_client::PublicKey(hex!("bb18de6ff89704f52022d088323752047cd0ebf8670930e153bcb6d4ed51dc83c18e4a314ec173e8967a124351e1d5e9").into()),
                    ethereum_beacon_client::PublicKey(hex!("a8d1375f9ea2516837196a47d08fbe9a52eb79c9707e31db7b97acf3f776ba04025b81834d306cdbf894defd9181cdc1").into()),
                    ethereum_beacon_client::PublicKey(hex!("d8e8ab56a33db91ccaf7cd72c18104cb5fe4a31aa9e1d130ac8cf5db32a72461b6c02161db5e0b7c9f65cfe72fb1e1c7").into()),
                    ethereum_beacon_client::PublicKey(hex!("7a66d617a402e671d7b3a49b237716e1306e71c2c8cabd97468797205549565380d941c978e09719e5412cabbb5e04ec").into()),
                    ethereum_beacon_client::PublicKey(hex!("3d0574c09ef94ed797ddfeaad8f41220c7ed7ab480e77fe8d8c63a5b034ff865f9ca16ba02051f58eb8571753152132d").into()),
                    ethereum_beacon_client::PublicKey(hex!("6361aa7f1474475631fb92ad48cd0482048bf72de7bbadbf2ef89b9ceb4ca3a9e4465832ea8ca5f7f7eaf6013249f89f").into()),
                    ethereum_beacon_client::PublicKey(hex!("d149bffce1398c121f5ed9f513ec95032fe0ddfdf50345f0dd2bc728fc44931212616b2a55c2e63633a9fc04b978023b").into()),
                    ethereum_beacon_client::PublicKey(hex!("289fa13f9df0e6248bee1bbfa97fd76acc9efa8c6fa2f5e1a879a469017d2dcdd3de19c95b221f97d56796fbcc698fd4").into()),
                    ethereum_beacon_client::PublicKey(hex!("974f3db12fab335af96fa2d01001017985b275b8f3256a7876027aa6eda3cebe84da40174927fd7473067af25899af45").into()),
                    ethereum_beacon_client::PublicKey(hex!("a300a56297ac25e75abf4302418b31600507ebf8c363bf7a02797c0d0b3d9c8486c78c8a27435501875bb132bbedd0fd").into()),
                    ethereum_beacon_client::PublicKey(hex!("0c0571e571d341a5eb35154ba00b1e70a970473164c877755a45d500edffbdc14e989fd74b60f7b20ab698c1b5665c3c").into()),
                    ethereum_beacon_client::PublicKey(hex!("028ff156516b95616daef35d6ebde8c97adc49f1b47e5e1d37b7190422b19c54e25eae492e0a08ecd4e57d0b256a13de").into()),
                    ethereum_beacon_client::PublicKey(hex!("6c49980ca82c1d0698d0b96fe122e8f0862b1cddbbc42d225e4f11566357ff3101a112386a3d10f66763750f06db26ac").into()),
                    ethereum_beacon_client::PublicKey(hex!("0c2336c7e90bab593b0bad69b08fee00170532e88ee4f2457b67bc58a5dc7372b16ef6a9ac06adc38f4d0b2af7e24385").into()),
                    ethereum_beacon_client::PublicKey(hex!("8e58db501e1bab0679a9bf2f0e334b1d6c02b9299fd3bcc86fd5f0237a7c762541d2202518ea533558ff346c3a04b95f").into()),
                    ethereum_beacon_client::PublicKey(hex!("bea17ae6fa254e824988874a16610a2a1855bf3a4cd906ce51d70efb320bed0d38d2fea4e686a1c843a79135322da2ac").into()),
                    ethereum_beacon_client::PublicKey(hex!("5555e6d620cdab05722518e1425a7a1a7b2bf11e66256a4102311c179a7a641058fcd3f7d3472f0f3bb63a6b869eb185").into()),
                    ethereum_beacon_client::PublicKey(hex!("b31fe8175f233d9eef4ece8bee9aed74c398b6a33b4e0763f3ec8f6a823dc06504f83ad6efc0efb4d36638752b6ef9d7").into()),
                    ethereum_beacon_client::PublicKey(hex!("d71b61fed4c5180aa3db48050d26910b85923908d8b008492c0ce79cb537876ab4ec3e5502793ecabfae0ba0e7b186fa").into()),
                    ethereum_beacon_client::PublicKey(hex!("9ce91664331a36ff4de3ef5209ebae7929fa9ea9a7e4b16659f672cbdcc258ea44cdd37ac3111e87413c07b707577990").into()),
                    ethereum_beacon_client::PublicKey(hex!("7c4f8769855c0025a645d9e4cc3715cacecab680efb6022dde8680241b9c60616aa712307923d909f1c8a0e8c78cfa35").into()),
                    ethereum_beacon_client::PublicKey(hex!("d4a4f235827663f9a59739749c0c0309d27b9322c314544cfe82b55ab806f6fb4915037e6bf63245d3b16812c3fcbc78").into()),
                    ethereum_beacon_client::PublicKey(hex!("917f3a5749a5e944b2b84b2a863a98bd7020b97e2045639229543c725314bfdbc54397d629cb34bc8715f60dc0d0e01c").into())
                ], 
                aggregate_pubkey: ethereum_beacon_client::PublicKey(hex!("6d11763ae7f45b8b77916988126e200f7be7f754abe03a27134456f8a1671ae172eddf182d185ffacf557d23ba267ddd").into())
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("24409c991386e5d43bcecf871dc1fa395013f0293c86766877f745a408148a3a")
        );
    }

    #[test]
    pub fn test_hash_tree_root_fork_data() {
        let hash_root = merklization::hash_tree_root_fork_data(
            ethereum_beacon_client::ForkData {
                current_version: hex!("83f38a34").into(),
                genesis_validators_root: hex!("22370bbbb358800f5711a10ea9845284272d8493bed0348cab87b8ab1e127930").into()
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("57c12c4246bc7152b174b51920506bf943eff9c7ffa50b9533708e9cc1f680fc")
        );
    }

    #[test]
    pub fn test_hash_tree_root_signing_data() {
        let hash_root = merklization::hash_tree_root_signing_data(
            ethereum_beacon_client::SigningData {
                object_root: hex!("63654cbe64fc07853f1198c165dd3d49c54fc53bc417989bbcc66da15f850c54").into(),
                domain: hex!("037da907d1c3a03c0091b2254e1480d9b1783476e228ab29adaaa8f133e08f7a").into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("b9eb2caf2d691b183c2d57f322afe505c078cd08101324f61c3641714789a54e")
        );
    }

    #[test]
    pub fn test_hash_block_with_transactions() {
        let hash_root = merklization::hash_tree_root_beacon_block(
            BeaconBlock {
                slot: 29667,
                proposer_index: 105780,
                parent_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                state_root: hex!("6dd515216ced0b701fe5aff4fbba95657018962916d1ae81dd27e477e92e248b").into(),
                body: Body{
                    randao_reveal: hex!("9120246e08d7876b3f056a84a6dffe3805b4870b37ed4caad38fd36632f5b2d9bd319ed71fb6ce470af8fbe0a4dc28ea15817ef21e93f60e9a5f3a1fd2817b4a9cc9f13b39ea65a74341e8107d71d7ac3c4da68e6a126bb0a82afe7bda4ec156").to_vec(),
                    eth1_data: Eth1Data{
                        deposit_root: hex!("a7013ac864a2d1a98436566218963bb0e1063f484419e11ce05f2ae899721064").into(),
                        deposit_count: 2455,
                        block_hash: hex!("a7013ac864a2d1a98436566218963bb0e1063f484419e11ce05f2ae899721064").into(),
                    },
                    graffiti: hex!("5421204c69676874686f7573652d4765746820f09f8cbbf09f909de291a10000").into(),
                    proposer_slashings: vec![],
                    attester_slashings: vec![
                        AttesterSlashing{
                            attestation_1: AttestationSlashing{
                                attesting_indices: vec![
                                    106042
                                ],
                                data: AttestationData{
                                    slot: 29174,
                                    index: 13,
                                    beacon_block_root:  hex!("176310da1eff663d901786ddd4846de168f655e5392e7cadb26bdd05e98377d0").into(),
                                    source: Checkpoint{
                                        epoch: 910,
                                        root: hex!("6d90b287e690fd6f8941f026578274da939097ed91ef58cecb894ae77db834cc").into(),
                                    },
                                    target: Checkpoint{
                                        epoch: 911,
                                        root: hex!("38548940e7fb08579b5ae46be764c9a3db6aba275330658cd27771c55c4c463f").into(),
                                    }
                                },
                                signature: hex!("979b66bb70fa8cea7ee829fc70dc13492c6db9ea9f97112696ea803b46b6348e8a3926122e3a4ff3d4ea153896e54a9f190db7d3a58f58edb69589025c18733917d4f1b21610eef8fb3160875d279a7ba37affdc8f5c6e21923ad97eadc7f308").into(),
                            },
                            attestation_2: AttestationSlashing{
                                attesting_indices: vec![
                                1427,
                                1592,
                                2728,
                                4061,
                                4843,
                                7371,
                                7755,
                                9329,
                                9804,
                                9923,
                                12835,
                                13061,
                                15896,
                                16114,
                                16559,
                                16834,
                                18212,
                                18265,
                                19420,
                                19547,
                                19613,
                                20349,
                                20463,
                                20822,
                                21103,
                                21500,
                                22066,
                                23008,
                                25999,
                                26305,
                                28185,
                                28606,
                                28895,
                                28896,
                                29040,
                                30487,
                                30780,
                                33302,
                                33978,
                                34552,
                                35727,
                                37094,
                                37571,
                                38404,
                                38629,
                                40143,
                                40712,
                                41209,
                                42948,
                                43204,
                                44096,
                                45730,
                                46021,
                                46538,
                                47177,
                                48751,
                                49130,
                                49911,
                                51327,
                                51786,
                                53818,
                                54298,
                                56147,
                                56379,
                                57511,
                                57598,
                                58171,
                                58878,
                                59325,
                                60241,
                                60992,
                                61532,
                                62324,
                                62816,
                                63282,
                                63455,
                                63813,
                                64064,
                                65111,
                                65113,
                                65237,
                                65348,
                                66288,
                                66419,
                                66784,
                                66936,
                                67365,
                                69650,
                                69701,
                                69809,
                                71607,
                                78039,
                                79103,
                                83588,
                                83613,
                                87810,
                                88410,
                                88460,
                                90714,
                                91202,
                                92414,
                                92874,
                                93013,
                                93440,
                                94839,
                                94872,
                                96821,
                                98647,
                                98990,
                                101140,
                                101141,
                                101979,
                                103067,
                                103850,
                                104018,
                                104987,
                                106042
                                ],
                                data: AttestationData{
                                    slot: 29174,
                                    index: 13,
                                    beacon_block_root:  hex!("f87c9c8ec942f82d24776c90756aa602cad967777e4367aa98e107c75ebcc8e0").into(),
                                    source: Checkpoint{
                                        epoch: 910,
                                        root: hex!("6d90b287e690fd6f8941f026578274da939097ed91ef58cecb894ae77db834cc").into(),
                                    },
                                    target: Checkpoint{
                                        epoch: 911,
                                        root: hex!("f337ac031629ca832d6880e23ceca9b693abc30448d7c4013d92588b9cdc5614").into(),
                                    }
                                },
                                signature: hex!("aabee22c3beecf0e1d3a663eb681728eb9f1d965957499567bb4e9b5600c800cead2e623ca992b2cb557745d591d59d317eb8cb871ebecf20952fc1f22096a12dbbeb7eb94ba618e2b45b4c30ce842e831d55597a4942910dc8afa17c43e71cd").into(),
                            },
                        },
                        AttesterSlashing{
                            attestation_1: AttestationSlashing{
                                attesting_indices: vec![
                                    105470,
                                    105825,
                                    106107
                                ],
                                data: AttestationData{
                                    slot: 29389,
                                    index: 3,
                                    beacon_block_root:  hex!("cbdd3159d67541eb599f47666cee2f102f0c081709b41b954a91296bb6d9e4f7").into(),
                                    source: Checkpoint{
                                        epoch: 914,
                                        root: hex!("3d838c21872ac53ee98bb73a546bdc7b784d21c1951b6f6d633a3cbb8862dc24").into(),
                                    },
                                    target: Checkpoint{
                                        epoch: 918,
                                        root: hex!("cbdd3159d67541eb599f47666cee2f102f0c081709b41b954a91296bb6d9e4f7").into(),
                                    }
                                },
                                signature: hex!("825d696249fbda2a53cfc95833ee68f42805791b6586760584faaa7ffd3f18d62fba159f99febb90a575bf6681c6c80501804c70bf801e1647769c696d619027950f3a90da1bb0c18228c703dc8239f81a061bfe5af6e58a2f866bf387cc6af7").into(),
                            },
                            attestation_2: AttestationSlashing{
                                attesting_indices: vec![
                                    741,
                                    1271,
                                    1495,
                                    2320,
                                    2512,
                                    3639,
                                    3794,
                                    5111,
                                    5339,
                                    5804,
                                    6009,
                                    7508,
                                    8006,
                                    9135,
                                    9196,
                                    9367,
                                    10273,
                                    10353,
                                    10753,
                                    10796,
                                    11214,
                                    12381,
                                    12445,
                                    13349,
                                    13474,
                                    17594,
                                    17599,
                                    17939,
                                    18419,
                                    19234,
                                    21523,
                                    22449,
                                    23479,
                                    24256,
                                    24839,
                                    25492,
                                    25928,
                                    25944,
                                    25945,
                                    26370,
                                    26493,
                                    27398,
                                    28034,
                                    29543,
                                    29680,
                                    30150,
                                    32241,
                                    32288,
                                    32692,
                                    34515,
                                    34757,
                                    35048,
                                    38651,
                                    39406,
                                    42276,
                                    42683,
                                    45748,
                                    45956,
                                    47531,
                                    47606,
                                    48011,
                                    48759,
                                    48938,
                                    50577,
                                    50720,
                                    53902,
                                    54180,
                                    54569,
                                    54744,
                                    54805,
                                    56527,
                                    56649,
                                    56783,
                                    58923,
                                    60385,
                                    62340,
                                    62475,
                                    63132,
                                    63463,
                                    63927,
                                    64147,
                                    64667,
                                    65052,
                                    66017,
                                    67750,
                                    68488,
                                    70564,
                                    75651,
                                    78815,
                                    78870,
                                    79539,
                                    79830,
                                    80262,
                                    82412,
                                    82840,
                                    83700,
                                    84532,
                                    85335,
                                    85958,
                                    87647,
                                    89656,
                                    90187,
                                    90598,
                                    90997,
                                    91014,
                                    91885,
                                    93040,
                                    93549,
                                    94103,
                                    98572,
                                    99757,
                                    99993,
                                    101065,
                                    101468,
                                    102954,
                                    104991,
                                    106107
                                ],
                                data: AttestationData{
                                    slot: 29384,
                                    index: 18,
                                    beacon_block_root:  hex!("ac35d537238e536a1f07149f23bccde9a986d753a6aea7fe76507db9dcb6df06").into(),
                                    source: Checkpoint{
                                        epoch: 917,
                                        root: hex!("829b6572ad21760bcfd5226e1add7994ca4821c91cdfc8d34eab3a9d91660ef4").into(),
                                    },
                                    target: Checkpoint{
                                        epoch: 918,
                                        root: hex!("fe45cb980f64f7598e58adc6796df579bf5ffeefc793814a55f21e8e22c4b861").into(),
                                    }
                                },
                                signature: hex!("b8d24df761aecdb5abef3275e4bbdb0151eed6f0a0aa8572f7672eed6c13687f49833406d9aeceac4743ee4fb916c5b517daef492c4d90af4ba6800d1bba82ee3d3ae13f442adc116d6f0d7a0cf21314ad6755ef0b32dd2c12574938f4eacc86").into(),
                            },
                        }
                    ],
                    attestations: vec![
                        Attestation{ 
                            aggregation_bits: hex!("df5afff7fffdffffffdef7fffbffffdf39").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 19,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                    root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                    root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a8722be87267f940887f57f06d747f02c0f845a1320ccb80080aecfef085e6d0bcf4778797a0a7342ecd85327c3f404214222a7bf70f4e3cf8c01e05c7d7ade74f55336003f969effa8bfca32583a00c36a72f58efe718610e2ccded694cf04f").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("f3fffffef6eff5ff77cbfffdfff6f7ff37").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 21,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("861adf03fb46b09e636e4a3c2e2d51324bcd43f7f33cdf6714d2c0ed6101d46cf9966e4348ba91d2152429c1c5e2c9d21154b8ac20bf552f03a38df20bcb291fc423be65b6fbc2a72179ea57e6a98011d6e71349c717dfd553c12180d525bf27").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ebffffff9eefbfbdfbf5ddfbfe3fffef3f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 4,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b93fa4bc648b560cb22afb22afd9bbee36e5c017fd7042f647d0a641599aaa9fd9c332a7ae021e9f7a52b96c48bc767c165f15e4d4fa7fdeb5e0449c7f9e6b02ff60758976f6844df8e4cffe37d116224fcaaaac8f05c6979475f4aef387dce4").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("fd9fefebf7fbbbbff9ddfd5eff17fffb3f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 20,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("9495517cc07a79d1ee525a41da1dd880706c449cfe8c29a936279189266da225658d44311485972aebd07d15b7833fe901bb15a85bf92b2bbfc8783855c71c32840cc8ddd20da5ee4976b106ba79f8ee8231f91b90b4a28f772633221e1a08df").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("dcd7fbb79edfffefffbbffbeff7fe73736").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 15,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b42fffad2349d33c1ce85365d4ab627ccb807d450b1c47e3948fc047de9df976b7b9939ac0aa931c69b15e55b152c43b0f03324b5211c93f2e69765980095f753fee44efd4382f5badaae6668cd1eeeb0060657ac33da375dd2bd7727629c174").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("d7bebffbfbc35ef3fb777fffbefffafb3f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 6,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("82acf37db996d6aa42d792b2feef149bb84806f368cdadf4ca91b7345fa2cdaeadaa92359e78a2a49f9541dd02af10370ae2a88e00d9bcd70bfe380e357e81466bff74f581bf75a77852b0b8812769d7e91c6278725ae8e31ff188d26dead89b").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("dfbffffbfedbf6b9de7eeff32fdd6ff43f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 16,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8f3194930a4aaee146d417864cd361ceeccf2a8850f31e2814bd55307cf70caa531b8489d586b2df717274652354bcd413601dc0dd37a1998d6123abd5c0a16e134e59ea0a7fed23281a976ce4aefeeb9f91b2f333ccd5fe7351a2f3ef360344").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("fefeffe85e7ffd7df79f3fb8de7fff6b37").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 5,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("ac511b3b57c642c04e4c58f2f19fc14d01e7e36153ca482171256e49086e08280591a618174ce66426e576f3e0969aa910dbdabb0066b0ee3014a0a34716aad61bd997edeaa2de817eb8632940ea097184c26d973d21e202a3813ad2d0c9f640").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ffff6fe7bff7fead9fd5bfbdcbfff3452d").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 3,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8d6847884fc5ba03fd28713f06cb980c373f4ef5d6fa02d93295bb48f57f1dfd1c9ce1effc8bc23153d527bb358e710e159c887ccb451d5dc111e61db28b34a8cf1a3a24f48076de77cced1742e1772cd81a294e5b3809c3a25607c9c88ac682").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("53efffffbb7f5bfefdfbff56a7f27bb52f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 17,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8091bf5aab53e117c44b57494478d47c25a62d4527771be2a909ea8ce8ffbcd4f2209554d11827c126f82510bc3d566414be1b1e9825b557b6f67e2e91d5c96a803b3e89e4499d745a67eccf808e48224aa53eef62236b24b468b6fa4121f708").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("aafd927a7ef7f73eff77dffffef7fb752f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 1,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("aa48bc603b860a09b78f42525a181514849e7f18cd423730fc5e2ca57926e429b86a133e789c8b31faba93d7c0ac7806080ae5de6c329371af0f1b7d9dee3143e15c4943b76088f1c2a45d239c9666b32673dc7f910a4526ed158ce1724ba940").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("bf9ff5ffefffd77dbbcfc5ffe1cf7b473d").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 11,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("add023642f195ef6edf99ba7316bbc5b74686aa7985663a885a71a621f315f9b8df131ba866812fa779d64fee16e33c801b5caa9a595f8583c7c9a8d601ad52db9c9a9499a6d55240b4f581a9cc33d4f0d5dd8fbb6ad610118e1b567c134b213").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("5fb3d7ef9bfef2f7fcfef7fbef75aef537").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 18,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a7e725b1f116fed35382d13e7f2120969a156ac5ac2a044c74b2cda9e544ecc91d836de172a6891090bcf6c1f303e3d500eddf37ab23947d1bfff9d6c9c9636497f7d7c8e2e3de88ae18438beb9e1d2210b97f5089222655bd4384bd00eb2afe").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("03ef943edffedef5defffe7dffbeef7f3e").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 23,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("90bc19b89b2be8d0fb8aed72f214e74e98a676cc9d248813729b3acef9bfde9fedc616e05dfc2f8e3e81db91c8da442513520684474dd99f710e8d1249a4d59e8a8bec10b93f196f19c70fb8e35841821cdec924ce0842c670c995a86fbbd86f").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("f3fff77bd7fda66ffb727de7f96f7ea93f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 9,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a1c5ca897d9c56bc686556b28eb9bd94864332cbfcfd6be7777ab7eb66f899a0a474c4d7329d1bda3e2a6f5e221ffa5815401d9f4ac2f7d74fc52241d62f08433be2b32cbccbfc180316a5f3c0373df2d299fb848df8afc9d284ef1a13893b18").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("69eb95d727fffffff3937dfcf7ff8b7f33").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 22,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8d9f95f5eeb464430082ae06cb04448c3e6a3b1c8717b7b01b8353d06c20e141ebbcffbddf52ea8c6aa49600eaf49cab0d953a8b65cb82ee277190c8d15423a3052bfab748de0d43b2eea003788a6df361b90aab3a221b56e81b86af004da83b").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("7efbcba6ffdf5376fd674f35ffde7fdf3d").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 2,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("ac6a0efe9bf3503942f19e9e165f6a55e87af809ac27a9911faffcc9bae3379deabe32a5d967b0ee9ecb5188c8bc54e7148c3563d8a194876a94e53a170ca4ad56b8d290490175a638ab0bcfc93aee3b59f448ac20915f29b8e6a8d160853a40").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("a77fdecbafdbf3b73fccff9f97f3cf3d3b").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 0,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("9117f01a42b012f37059257266893b936e1c2de839777d7e2d3ffe99b0ac11f3df3d16608cb62dfaa1072b2142d7c1ff07ff1b9100bf606a0e1913935a2fdaf80c125bd5f5ae61486e97d255408bcddbbb32734163b91cf0f5f942c8badfeb3c").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("635f6fff7597bb5fc7d7f8dfffe13d7b3f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 13,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8d4cea60189543f9547d9ec853cd5a233be18d4d4f8e275c9956d8faec7bd14df49c8487fc6efca3d593511cf6858d1807534d8372279954c268a82f84a141a358cdc98fa8622ccc3626405c9cb2a9eefe7a43d4fd1a99cfe2c16c26aa9a4904").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("fbe69f5bb7f35abe1cbf5ed7fbbfff773c").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 24,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b3d0b90fc2d27d5df8759d1b2d1c04dd3338e20b811e0c69694e7057df89d4425fe9a4e490493b972a5ca7c06ffa326912cb786ff1cd890fbb321822bf98c6b6330bee4f4695a9ecfa8b3c3b845c89ed602d9af3ad4c959501fc182073c57573").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("25df6e776fbfbf9f7aefa673f6b7ddff2c").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 10,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a63ffa1841d1ca031d3548beb2af5a9a7af80e1fe60019bdbc9450ddb41aeadcda463384d2665820089f20a3a23705b70081d77bbb1c8c4d12920a392dc47ca2c7ccc03ebff546723af8d9a29144eca0a0e9e48245380079bdfbc731bc5d47b6").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("673f63ff36fcb8d1b9ff16f97ffeb7ef7b").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 7,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a6dc3fe94f336c2a31bda49abb399e1c64aed896deed1daa26a9cac5d40af670dd09d49a5f80dcb4b227cd40f746b0ed007b5473c786e72a3754751ed754358ff9fe9dc7a60168ea831d55b497ef70913c8312e2f86e8e335fbfcad5f4597ac1").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("cd1ffbfc1bf653b7b5f376bef5ffe7d33e").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 12,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("af83ea631f3c9ed48d49359b535abbc033625d2ba3079026fa7000780adb93f8f08eaafe7648782f8f1b62c4f8ce2e6d0f39ac2843b13cd8b113e83a5c59d32751a1d68d9f26ce1283d0df4b4b3305c45c91704711a5804dd8581b6bf1526308").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("7bdfcf5bffafa731eef6d7bca4f5e73d3e").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 8,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("88c421dd36c40a826cb7871fd45a09f1f094383883008d83182d71d3daacdfccd357d04947a92a8f55714855d68345d116c7f65c3ee633a9884aab84f0b4c683bc56d7a66ae9fd98c303bda678c03cb2172f7cd183dec571a9dd1baa5315b679").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("a43bfaf51ab99ff774fbb9febd7477cb3f").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 14,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a543f4654009fec550494067bc215582f58ff78b7239a116c4ae67d6a5743795be87bf16091750face986c675d16db6a1764e53e8f363fca4edd442306e0f1ad7e09d7e2f80bb419e95e94960d2b9220e7589cb731de16a0eaad48792cffaf2a").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("7ffffffffffffff7ff3ffebfff5fff8d3f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 7,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("97d65a0b27c6abc3d1a4234a5bce1879b19ec6de96ed07f83e29241265fa2a8cddea02e315414fb07536afe9a2d09dc609aeb55a7cb9630c621713bc9f9b2e9c9b4dee73e636ec0153cd2d917fb650c76d1c96aa3471b015535fa3258a522cc4").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("fbfbffff5adfdffefafbff9fffffb3ff3f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 21,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a0b6cac68f44bb6656dc77973dc7ef9c684e58bd1c5f493dbf35c5c0e0f26ace2fd6318705fbc641a04ab16f9bb0e1a200a718fcbb0e543b028b2946b530a83a5a9dbb64d0f97804843d734dab446d20ccf474816512a9e2b82d427e2b9e5462").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ffdffebfdcdfefffdfefffffd377bdff3b").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 23,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8c0e6eacc216b3b96a4d594b12cf26ce0533ac5870e7f8f06fa783bd2d41cb12a9b0e63f8b104317ecda66b457de9a880e26c716ee458c6378bfdef2cfbc62d10a0b6cf714f8f2767caeb57475c1aa84fff439a17e1e5818cd722f43db1945bb").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("d7dbfff7beffdfcfe7fdfff7df7d5bdf3b").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 14,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("85024d49f4c997e19f683b55de582fa03d5a5ca409ea697207c84a383874b4764d8fa2657b1dadfa0bbf1a62d8067a1e1186f05418256dd33e0fb5f58307b05b2145ec79baae3f800fd5af1ef8e84ed2de0ea0223e64877fbc96c98660e60bca").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("efedef6e9affdff7ddf6defdf7fff5bf2b").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 2,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("824bcac9ab7361c627010d5527a7e9b0fd42b346122f3138d650c6fa202e9c03dd754913bf6fa65866a26dc7e6fe1cf11393fa195480e59fa07c5ca103a70fa43c5fc8c76967709811d94cec0e200031fab39919442be997d4fe87ba3245effc").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("deffffbfaef437fef7bfdfffe5afedfe26").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 5,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("99f87909f96c1ea644aecc267fc8fb5f4a0a2a2b326a9bc4097f5773462a694d02941222336300e4fbaafdd3a350e9b207b4c11f9f34ce8084787286704ab024ed5c46fb47061c13fbef842e45f343266f14d83b5bc30c9bbdbcf9600e5b8a91").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("fdbdd5df9febf5fbff777bf7f5f7dead3f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 12,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b8ec3c750c127a739d956efdd1d930574f51ba181b8b3ad7c2ec44cd30cfef8009e351875bf1d6f79a170d751c9ce556008e9316b42b42db9a99b93560828d8d3fc15da860d4c1f3ced06a55d670cc0d7189a253e21ed7ddf405c7948216bb07").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ffffffffff877f37dcad3ef7d9d7f9fb3d").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 6,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("ab8d1c471deffc33d20bc2a97f18a0060e74bc5ecbb76f454b8aae8155383eaf5f0859e923579bb2535b5b0d495ee2bc14b50823d9d971042e24fbe5349480b4814df15c5edeb0c28f20e1f80955cd7c48bd007f484cb2aaa3fe71c28d3d36db").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("f6f77ebffffaacfbfbd6773ef7fdbe7b3b").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 20,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b4506234b37f019cf71777fa723e776cd6c655533fee2ba13eedc97347b1f98fba3ec0b41e6d393275c9b3a97e58dcff14fe95f13d4adae336af00ebe2688814e88b2e518b18d4616c3bf218de845bd7d0172809f97b4f0e98696e55d968b920").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("7cfdb03ff6ffcfffbffbdffef177fcb72f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 16,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b979e84e297bf852dc9c525947d8ff2c9f3e3ca8debf4f807bce83855a87dc3765e142794a7d391b87db9d691f97515e0b94825ed5e03fa74a6ea7fde503c3bab9180af6152de236ee271a7579881ac3a9d587dd961466fbca28939036d2f6a0").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("31fd97c97ffffffffdbfadaffd7ddcfd37").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 8,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8afa752f94c86fd6247bb9b757dbb49e5b0eee0a37d1c71096e20129e986968bd9dbed36f481c8cdf3a6631c583c68da0ace6e71478b9c4b8552357577f5a5b643be554bc9dfb88a2c670aec110cf6b3b2c9b0f6c378fd307b17be91da80e3b1").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("dfeb3f6ef7f37bbcef7cee5fffb73feb3f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 4,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a4744855aa2c9206f5ea6783443aa7a69ec08d52606e694693c991edecd14d58afeab2c34fbebd37f16ca620576825a803971de5fccaf96f1efa3e980d686d9124ee601dc8f76694a4a3a9ceb72d122cdabd6e1295440bf6d37da3031e1b23f8").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("f7ff6dfd7ddf9eebcfd5fdf73eeeedf93d").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 18,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("989fe4e5e600ad53a867316d6c3980d6968fac9df485eb4d81f26cbd07e3964ddee5041f25589a5572d77ce46dc0a015006f165f3c0aa6557bcfa01108229b7ee532727cc4e703c3348086e33679e62e1a78bf1d95d474493f7016bdef4e8408").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("76e7dfddb7e757b5b7ffcff1ffcfbbbf3f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 9,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8a760117c7a09230723b7601b9a74e8255887b1ba328ab03f7d8ae29bd1ac3b5279bc61f41509be143318c43809766c30e3067860f9d580fd01663488443ae7f92c2d3b43d41bbe68e97f05a0f7227967641521a08a1bbb9a3283978435d63bf").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ff6df113fff3ddfc97fefff7b7ecbf7f2d").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 1,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a7096aeea3119f449f340fd0b176b9f9e125794cb1980ebe79f28b4edc21f955eaeea394832a4f3249063611bb4002e918d1d8c6e68bf69e9e6b74cf7394ab329dd7bba00d2969426e12ca77047bcab0be0ba7059a4e0043b0eaa6a1c426e4f1").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("73ff3ed7fb5fff5fbc7df3bffdc5df3e37").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 24,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("933a954a5eba70e60b997a6f59d3e246eb7e33919d088e641dc8da80ae8edd09d12dff249b4f34dccc84c458244b25c60510fcdc413a7a4d3d725dfda960c0f8040ac8134877d4a755d5c64fcdd1b417930263566f3cb7086620c1a73de75f6d").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("f77beffd79f73fffb9e5deff3c1b6fe53f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 3,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("ac1ef09d9146c1445a3daf4f5f31bd560166d6e4e74574394ac2213c503c5bc586b83ebaa531ca0f55af37da7d8523320ea1468e5d2f98b37bcd7d87a0b2b33916f2dd4d1702ef2f1725b501b0b1de6d4b6732e5ac4c9131c6ca83687fa3d03a").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ffb297f9bffdf79fdf7cbf9b97e3b8ff39").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 17,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("90528d6ab2050326dc6ce4ca63ab8360bc3d7a00e147edaac2f6e51cc9efb3ba169a95c3835184be5ac6c62967d20a7c0416a71d0bb77c6fca06ccedc4c9c460c41f820c12a869af26e3d90c3891582d82c210131f8ff1c5b5d9be737fe892b0").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("afe7ee7ffef6d479ded77745b5e7feff3f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 11,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("96f670307f19e9e3dd9fcc416761454069a61f7c2463aec2e5bed47666dfadb88d55b4385b79eb223b11259ee29efe751584c18e58c8754e3f80dfe0c7b3afdd3cc356c3e306a8c434be54cd980e9676bce5235776c229365467255a02ec8e21").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("a9797fc1d6ffbfebf7f5de5fecfcf7e63f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 15,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("957ee18f32b16eb3aa32b3177b1be862bc6e85c2ae51e03957c0c1f8fc93553909c5f0a69e5482295fcb87133dcaee6b04290d2dc1d3eaef27dde584f3360d605179a472827476662b7feadfb7163c6f7226a5e9a89ea1b5a534a48c77884068").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ffebc6dfabf33797a9eeffec97be7ff63f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 19,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8ff15e1bfc3958978e175951e298bc48860aeb3f3343f90cad088eb9936ef0af39f7c0a678eedacf4ed67c237af062a7066b8ffcbdfddfd72becc671a7bf148915208ebccdc0c3cc42a58d36ab7e1de181d031df672ef2b870d3f1f934d85de7").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ff6b795affd3dac8acf3f77feb7f7fec7b").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 13,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a919ea2f11cbd479c990de8a20c272019a65b2dc71821dfc3b4dfc6c8fc6a95a8047c7618d1a7e2afe03c486853ad5941580505548403775522cc3e5e1f43dc94869ecec3b066b0fdd2020bb157fce81c1132296d8cb1db9d5f0a2fc856dafa1").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("fadf2be7b177ddff5d6aa0bedfbe5efd3f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 22,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b7a0b665135cebc1d8f2251d1ac41f67f7c2aeff6f5673ff7993e25f8efe4b69903b582fe47be0ba4ff37c3d1a0ecf190603da6d560d685fa7cb77dc597e87f7dc182854d793cb913b5c45d6979632a3a52223adb7b3b780ffd8e4089260dfbb").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("fb1fedeb3becb07aafefde39ffdbb9d53d").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 0,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a9323181f2ec12d84b826f6a9028a6a27745eaddcf04af169a5da1f57e4f25687b1c3201e3568d3765ff310e05417a6d04e422ef5cf3a0605e947254d04ed21d20e5e4c23d028c515267bc983fcad08a94766e8d8d915d9c30bd500830be59c3").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("897edd6a37f7cfbedfaddfaf9bda331f39").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 10,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("854c04a3f4f4ecd4c1196fee335d3b46f73ec2b3381804cc013557aa3fa9c05895a182c9bebc1d86b0fd0da2105fe1320913b0c3183ba74f3619292925ddeb589e611360ddc16893610b87d0e26169da80ceba7ee3c69d44e4e6f26630205092").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("2080010000300410000220000000009028").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 3,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("906743637c35cf8fb1f28e6a74d3fdaa2092f660f76f8409ae82ef9928333bf9d9bc640fc7f7761d290c1022a48edc3f030c40d0244cade869a46ee43b43e280f9fefa52fb20cfb981b0965728af867109cb1a977c08be1039751d1f6f999533").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000020040030c04044810001801100924").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 18,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("949caa179fa0e7a7b78704573bca2ac3b2cda537f861590044962a7a414aa648930c03e1d7e5105d89e7151ca661599507b727b500b9806ca0fa5d83a358825e238f2fa66ad2a26b0c0f09ba5f32b11a98e896b8ae1824e05fb9592a23f5d662").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("4000200203404239104212048108300030").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 0,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b69ac93b6808d4a00b051e488cb329c2a7a062a2579ad47e71bb16adeeec56ac0563d11260d9ad01fba38e570c94924309b11231ebd67915c1ce0a609b9b2f28a0302f349f55a65af5c8a90a23fc87e81a22308e2e63edfcfbfeb615827317f4").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("4000010050084001010000000000000020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 19,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("aa98d78d8cc24e91611f5bddb1d0e94342864cb2681071bb27c6e5e2a0dc100b60fb4bce37bfc52c9f33f55952bc4952176dd55abdb1415f4849eb472d4a2c8edfca5e5ee1bc158fd7f14d2227809c1733c3cf19ccd2a58d2a66591782854ea0").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("005010030080000090014141c400250024").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 13,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("953f1ac2af9a82063c9b321aa1c7d3f313381b348b7cdf4f4a4f4d33f5e002981890d93e8a8f89aa99212ff3ac0078f3047186a05049cbb1fd5b3ff5219aef19aeb3191450f75bda173e61970bc3720e6c5084f6fcb2cd4438f2915dbee798d0").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("8424928002f41252080104a00023008224").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 12,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b3633926924082359929fafef8fb4de1e55853421e27b7879cd3dc1dbd73da677ffb07d6069df6c573cd1c3cf333433102ec17099bf50cd9be2386b3f3765ed02147956430ecefc77251b64302a941765be3a05b3e98f34dc96887640b511ab4").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000000000080000000001080000040020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 6,
                                beacon_block_root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8de6597bbc6d569e6effcd45ef19aa7188c6663e1b3f332333206e55effbb565f0512d309e4706272a824e20b2b755b017680a905a051212ec69723db614dca428a24dfff2f8bc3027e5bee1069d2876792ccfd6aa8751d27a0f880592a17cd3").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000000000000000280000000011000020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 1,
                                beacon_block_root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("ad3bea32edba89d76919d5b397ff8f215e11e9252d5ba3e3c83856767a1e1aee733b775560739f5814843cb590e079751527fce5a7dbf8e11bf9b189a29aa07aff3c4c17fdec5c696dbde06c8d30e05c0fa39a098ea79c9db0b2eca27e90cd80").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0404000000000000000000000000080020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 21,
                                beacon_block_root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("87168e1c83202330a6d58dd1387147acecefc603fdc5bed9e59b80e63d19eff6b7c0114f2c5a4b0df4099bbc0e94fdcd02b1c75e21e3caa911680778974d2f5e8b6b7867af375b0a723b4301161d523ed69c0ad0e04dacaf4e20bb12b489a2cf").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("00a820080a002200000010208840a00020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 20,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("882a41bea7e03ff7e51c789ef5e4a8680e97e500acb9fd477805a223cfe34a698c851ce0c50fdd40ad20eb3e4f42bc52071126f33ab612e289d05780a7ba23ee520c24885a174a7589b0eab4fc0f4d22501d8550cf580efefdd48c1374654018").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000146600840384a88dc604d020100124").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 4,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a25b6c4295dc39ec5ee4ca8d078d8b3c146bcb27f902dedd6e110ae574481b3342e7ecd49b377a42f2934b9d3cf2f62917fee6e6dcd22eabec765159be2bf8b5adb03ddb2987ed2c77f5f05b5e24e81c25670a80e460007bc256d1850af3804e").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("73e799cfb541913093c98df0fd4032a22b").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 9,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b99b37ea5eb04b0cd86d7292619885dc3629e80d2cc03699a1dd98316025a08a90a39e0f7402aca7abc9710719f812ee03a4d8e96a11f50cca2ecd10b2ec5b22f2cf7007c404bafed2ef9987080b2f5bb67b3381bccff998da9e2a7186056e4d").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0030120000804040040002001005003938").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 0,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b9580ddbe2e321795edda9e85f64c1e5e64da78205919d7fa88f2d613c7c0181376808e6a7be8361704064cdb7f2edbb0cf4c3b8f7023ef7db1a4692b8ddc2a2ce752c575644db357d9a0b4b3bfa66ebb11f9890319f4f2a242e11b6bbc4e8d0").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000002000000000804082000000800020").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 11,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("932ab45cc85f93e6b34d07c919e1b4fa576551adace66413ca5c462d2caac43d83297847d72135d457525071705ece2f18c6ac229857c18460daadb6d0e4e791b93ecdb4c1b60dea76db057f8efd50674bc744a5e214ff7c2f19bfa247565bbd").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000000000000000001000004000000020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 3,
                                beacon_block_root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("acad6bb4a0301971c3201c74932f86594e82e7c4a4bbb6fad1f7e3afc2b4bcef2aff74e50def71dbf8ca8895bc05f4ec0cf728067247fc28da5a8d303fd33eba34011c35a917e1c592a4ab1cea7bb3e6bd4c445aaae76f486c1c16cae7b084e9").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("8000000000000000000000000020000020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 9,
                                beacon_block_root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("af7c9588d16c29f69bb7f382204315c26306ef8867dda8719e38b47fa9e540a67f2b3151831fa836dfd5f37abcaf1bf10066a455cc475a31136c5164387ba846715f009cab0c1ca906c2d616457d6cfdd9be76ee8a4bbe0482fb8adc2a5c3689").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000000000000000004000008000200020").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 5,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a6f4ff58a3338b22be3579995f07ba57f2ec7708a5f5d5ec10c44fb7cbeab263c1436217f247f68ed9a8615e7dba568807cadb4ce903b2d1f620de3224fe8e8683ba7380174112bcff2c5f6553288fd1b6731e1b344c7831f495ab246bae4e17").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("7a429e5bb1e35abc159f5e13f3165f3728").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 24,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("adec753850aaf875b02af5acdcb57103a5c571ae702dfeb692a1c962c1b4d69f7efe0aa60301bcadc77f5b0fb9f523aa055c9113800eaa955cc605046de254a4c999a51ae84563c91a59c44babed12d1b693b5fce1f86dcd90caef4cfdaf10cb").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0400080090000030500000000040000020").to_vec(), 
                            data: AttestationData{
                                slot: 29666,
                                index: 14,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("978799699a66382c964ca87aea73ba81e11d8f3edcdeb2d8ac7c4858b97d6300186b37ed84d300e0ee0d5925cc3ea5360ea6059b2fa82354daa8612a11f7e8fea051e7274e63eb6d076bc4e39ec3edfd73f9ad580c9f223fc9eed26f53c0dbdb").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000800000000000000000000000000020").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 24,
                                beacon_block_root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("b7bc906ee303c445948c3271302369f20272c4578f6152f32fe701d0daed9275b31a33f02f816b706b2b50e49583040c19a3cdf9d5d80086c0c8ca626dce481642cbf262efc210a6e76f36d2ba2db2c6e90d25a025fa186634b393fa599561a5").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("8800000048238000000082000000000220").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 15,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("83aad4160b5f8e658a84529f555f10cf97a356ea5e5cba2084e49f90af506f61b6fe6ab2cefaab850cb66a3c64a1e73618b7adc8f41ccb7a2742328b7b096f2b196c9fa4707866a226ca6b009e6254279c3fd64e9ffcb921da7eafc13509c90c").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("0000808c00808402013020000000203120").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 23,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("a7ddf74e1311ac6d26e28988c3fe51e7ee79873712d2e29bf34b0a2b9b2e36b5416692afc5b1f4d756d9cd091fb668f20cdae2bc32739e4e8d019e30ec5e05fb81ddb1f0df109685a3bee066c2e03482062de77009a8eb83c009122234f4639b").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("31fd97cb7ffffffffdbfadafdd7ddcfd37").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 8,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("97f9bd4c79e8e6411f6f252d6a36f804fcd567f2f6e2cbf6ba25978f6fd7fd39cb146c7e04db95931b2da12d32b1b20e033aff9a0ff5f0d5f75233427cff2fd7ae49e9e2f42a45886bb3602b171c1e26575b4ecc689a8d2bc5f74556c8ca3817").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("000502040a01040c109030180080100620").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 24,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("8695c54cb27bc638e80480c2d0417529e467597b1d6316eef97b3823d4ac57171a42461f95a923fbc0db34b4dba1b614108902401c8318de39f1c4737e116528c48487cff1b5fb9139a9fa60a6f6e9f3afb7c86d7ae957fbb1cffcec26b27cff").to_vec(),
                    },
                    Attestation{ 
                            aggregation_bits: hex!("ffebc6ffabf33797a9e6ffec97be7ff63f").to_vec(), 
                            data: AttestationData{
                                slot: 29665,
                                index: 19,
                                beacon_block_root: hex!("0a8d18a687af678f5dba0a7ff586c47b0fca5d04671cf7de076424b1af11bd5f").into(),
                                source: Checkpoint{
                                    epoch: 926,
                                root: hex!("fa1ef06bf0cee609087c790ab7394a49c1579a85c205d0686c098f3f67cd42cf").into(),
                                },
                                target: Checkpoint{
                                    epoch: 927,
                                root: hex!("34df342097dff8a8da8b8b2a34a67d15fedcc6de3d9759100f9ff54c58907c35").into(),
                                }
                            },
                            signature: hex!("98df5559893f8b0041d7c129e30c64eb707437bdf0833b116ff69aab202f043450b74b4aa5c6feb1cf77adb0435b24fd129516c81838342a2eaffe21c774d16c1486acd4817f6cbcaae7a8011d38abff0d5f129408d321cd41a476e6e48eb433").to_vec(),
                        }
                    ],
                    deposits: vec![],
                    voluntary_exits:  vec![],
                    sync_aggregate: SyncAggregate{
                        sync_committee_bits: hex!("32f767f59f4da9dfecae0defcbf76776ffe1beaf3e6fb1ef4be53bf613f9159fe95db7abf1e388fe4fabffcf28fddf913fd742ff5f55bb51bdcf6b6fdab3fbc8").to_vec(),
                        sync_committee_signature: hex!("965ba3055dd9a9bfaef095f98c8cd7dcad922451808b23d414c3035407b4f75fcfc7a393e19ab3b6cc8533992f931b9a07ebaf09705f524d8dfc3d6e567acf0ec58a78d3ca5f7dd3743746f2fe5e66c001dc846571726e44401f14a9a984e66b").to_vec(),
                    },
                    execution_payload: ExecutionPayload{
                        parent_hash: hex!("888cc9988c1c34f02d75aa19a1a49dc68df28b0aae2f1f1b59daf506de9ffdaf").into(),
                        fee_recipient: hex!("b204c525a4ece451fd42738ffe7fe738fbca5e7c").to_vec(),
                        state_root: hex!("c2c48a500796bdd5b2c48f8a6a7af1f6b15477fbbcbd0cdacb5a85bb756f5bb3").into(),
                        receipts_root: hex!("7ba27748bcd7b3d7c3e76414796b19874879d6b488c0ce9a1804b22958330b8d").into(),
                        logs_bloom: hex!("00000000000010000000000000000000000000000000000000000000400000000000004400000000002000000000000000010000000000000000004000004002000800000000000000000008000000000000000000000004000000010002000000000080521000000000010000000810000004000000000000009010000000000000000000000000000002800002000200000000000000000000000000000002000000000008000000000000000000000400020000002000000c00000000000000000002000040000004000000000000000004000000000000000000000020000000000000000000000000080000000000000000000000001000000000000000").to_vec(),
                        prev_randao: hex!("ae3e8dab1d1f7dacb8343f33152958655d7e8355c11f7871b5dac7eefd71486b").into(),
                        block_number: 55475,
                        gas_limit: 8000000,
                        gas_used: 1999728,
                        timestamp: 1647363504,
                        extra_data: vec![],
                        base_fee_per_gas: 1010580578,
                        block_hash: hex!("9bdec690b39f69acdd81e764b42efc4232de0c26ae4fb531e7e2b99f44f10bf1").into(),
                        transactions: vec![ 
                            hex!("02f895831469ca8301573c8447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d6300000000000000000000000022007a12f6494eb73165bdc5a475771ef2255325c080a088cc0bea0de7f99bba0dbf1dcbbde7be4aaafdcd7f55b3aedf0a73ad5c9d8d44a0219df1909da683994e6c209aaef1c187d1c016112ff5d1da975bf3fc44bf33ae").to_vec(),
                            hex!("02f895831469ca8301573d8447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000f1c26981dd8fd214fe9264a897d9e6cda96db648c001a072e224e49c25de6ca25f4844cb8419f2ff3109209e2a1bffbaa375f888fccdd3a00de6e29daff38a077dd311f807e63c0304bceca80ceb0d7d967f26ecca8e6682").to_vec(),
                            hex!("02f895831469ca8301573e8447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d630000000000000000000000004d170baa29a81df2877aa0a3fd798d4cbb92f1e8c080a0c31170b3ac41fe3c93968363cd0462e2e7886c9270407ca439d66914e72dbb41a0092c401b5334b5c4abcec700e8d5c5f31915e0da3927442a743410665fdc3cc7").to_vec(),
                            hex!("02f895831469ca8301573f8447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000be7269d183e23e093bf295546a63bf958db8f65fc080a0ebb003745552875567fb0ece1d2eaf8e7455de9a7389d54056c8f36d77edfbb8a00d3d597a0dab9e91ae0963bebdadc5e10a7d892522b589c26ccb43bb6cb657bd").to_vec(),
                            hex!("02f895831469ca830157408447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000f9a13d1ea489df1f1a8915436ad3f4930c6d1686c080a05c9554f59f545b40cf7690f72742211ae64a4d756e24f333d016127c851405e2a0370d6de5fd03fbb0d3acf6eb1eee5dea12febf1b584c2572240bcee12b5ab945").to_vec(),
                            hex!("02f895831469ca830157418447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000b00f3c318685b1e6bebecabd06578dc00fa1dd73c080a09da5af704a68e9a4f2046e8b1d98d1a78bac4b1ad0a52e767c0699c8985b5133a044ad453e47fddede101f2fea0717110aa784143516c1ef1d356f68f88c28251e").to_vec(),
                            hex!("02f895831469ca830157428447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000b0e54b7088acadbbf4eb35f0db7e35b08bce340ac080a05f83c113b7111864e495f59f9ba8d1f0cda20e1d8da8abae7846001cda692ad7a0388f16e94959934d9ed3e636ecce25cc804f477204e6043d37488c5aa3740f89").to_vec(),
                            hex!("02f895831469ca830157438447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000501f11a1152bdd74048dd1dfaeb73dff0b1c8936c001a0dd36891b4e17c8670325a70182d03b643553fdd413b868bc148c8c8e61571b5aa0359d2c6fb13956411a7cfedc6479c683d98b430c42b4b483869e41bbeed777cc").to_vec(),
                            hex!("02f895831469ca830157448447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000411a1b1909deee4b60a5bc16d6c2dc35bf0b77a5c001a02fefaf366775a68aa5761468d0b928fc7f9574df2c9e3b86e5e009f5d23b7358a0539a850d3378931791bfb14ed62eee0971db35fc499eb510bc9798589c5333f6").to_vec(),
                            hex!("02f895831469ca830157458447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000f8da77605dabf5e0682e8eed9020fa1d90ddd916c001a0f5021954633e54f921201ec47c759506b606412c6ebe97bf7a434d59fa1fe31ea07c0532227f710a9806d03d46d5b7f6cd213a88e3a5c88845312df1b30d43a096").to_vec(),
                            hex!("02f895831469ca830157468447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000adb12988cae14f9170b6f38819736636a1fd1290c080a0bec0f64d34b8f71fc0411020b4ee3f42bc2708472f32e288a6e943e3ade2d9aaa03b15f70da19007c1d16c818e0eaee1f384aaf3210f9a5beef92fc47cc9682d07").to_vec(),
                            hex!("02f895831469ca830157478447868c008459682f0083030d4094871d96f0d74b099ea3c8b7a2656ec06da1ee7bf680a4b86d1d63000000000000000000000000b731c7d4947c5cc24984f87814878cf0e131117cc080a079b9500d44b92f9437f7e69dabd236316b2e2d0813a86ef734b3ea14716aa03aa041cd5ec2cecbdfa28ef356ad6f568daf0eccdec5f10b58d2185ec235ab88118c").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc712e4300d828808328d3b8a009af4ec9a9176ea4c608107f1d358b89ac2bfcb2bdec77df4b9064000799a354a0626f45ba9d0bacbc15405f2f2b273ceaa25bb0c2c824ed4d1321a02a85f8cdea").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc712e4300d828808328d3b7a05ce94521cbe0adefc7ed375724c8992f197ca0ea70440d611eb01d5fd18910d3a069e57b6591e7c68c005588430b695b73de609ff8845092e678b6d1a1f4b42912").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c314789037a896325d878d828808328d3b7a0dbe22575c74be7b99ba904f7526512952b5f42ebda5c8a9c13a459a429f7bba7a003cade1a06acf44daa91262439755b6c3efc2fc13e188f468249f379ad3e2edf").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7f36ad88d828808328d3b8a010b810848a6dc60a079fc011689c921ac95a1d50a92e4ac885d5a2b79e5caad1a020b0c9d9bbeb93949d550bdaca6eec1daff334faab631b918cbe8cceeccdb4c9").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc712e4300d828808328d3b8a074142019800a53ee12335d6f9ed43bc43433ec8ca5171579235238aed83468daa007ce4e819ff6cc2e6b99ab68449aa77b095fdef336afaafae5e1021ce013b8ce").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228d3f1810d828808328d3b7a09fee28e48c07b9a44c5463126f6dcbeb0a0e6314494495e782d3f857c945bf69a064a145cb2a417d088262dc8e67b61b083e974679cdab287dc09fdb04225f21ac").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc712e4300d828808328d3b8a02b1967ed69fdaf01c205135e58ef551e815ad354459c2ac37a7bb33b56635590a071146f5c5ab14cd812fe308c397283175bd51635252afb81decfab1c5677b0f9").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7f36ad88d828808328d3b7a04af604b7a480a2d8b50ea929b986d820b28b347efcc229bfa801748784e002d9a04f9c34401f1d45bb449df6cc7a621b64bf5ebbe35efff6d532ad552e241eebd2").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7f36ad88d828808328d3b7a0be934f40e6f94b360f05b54e9078611c6bbf1338668722869be8ad24d1ed0232a00a32dc8e95a46e21abab76532ba38cc0dda558563881988919b087276f4310c9").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc712e4300d828808328d3b8a0b0fc5bc56c9edd02afb2c071ceaf1beaa5a57aad6664b75a5ee204010354cb24a06a2e888895405828d215de41613b34f2ff97174de94cdfaa7480e72608264f30").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228d3f1810d828808328d3b8a02dad810fb25f6cac0c4da34f9876f21078a3841384aaf811968a2f84f5ae83a9a0318527b29049e794a97524e85e9cea4a4db2adf1ead1434f7a34e52a3b589fce").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc712e4300d828808328d3b7a0a79ee7e1d828efdb9c44ae89fe9c969f202d37ebab6ab0b2a0a36ee2dd5b458ca079a4ef52c17d6e561f26df4d5767b22d7583688915df6ea3d69d77964b71a899").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228d3f1810d828808328d3b7a01032720078036793a767c4f6955d74c2d445fae3b086c962d4d297c18f1503cba06db7626363d839fef91a030a2a70d46b680a91086319c60920ee5c6f9502d6ca").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc712e4300d828808328d3b8a098ff2f8c6f8032fb50c06bc7e2f338e4bd3f395f98173d62ebfc325325e4a565a008f365fa9843940a7ae28a316e48f1c670cc420286aba898ff78213588f4af28").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c314789037a896325d878d828808328d3b7a01ed7a805d651e6093033626b3a62b69f54be41ecf3e7f39ba488cd950cac530aa03f111a5ba0631d795f5ba1e9de0a0e0bf54f27dd91ad67a48b32e50e5b4c589e").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228d3f1810d828808328d3b8a006bfb96ac86a4adc65112a5d40ef6fad2bc41811798161485145cade32ac035ba07b3173029397310489025105581e0119674db092b4a532fd237f7ecaad38e3df").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7f36ad88d828808328d3b8a02e985de7efb5da3bc30184d776a0ddfefc0d5c3de47f2211c10dc58a759a3fcaa071f254165a295a701d4f70ee411d47c87af72612762ac052abd6a8067cee7b04").to_vec(),
                            hex!("f86f028485bdfd3482520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7f36ad88d828808328d3b8a0f6f87238a92c29488e9a18ac1e7fd78297346848a0d1e532923d0c49925d64d7a078dfde5758e215b2026b9518920f0df8a66fcb8466153ac8ae58ce782926fd9e").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228e041603c110808328d3b7a0f72888d93475eb27377b8b761284412a672e18ac71ddb8ed9b96894580f72c46a05516df11f23ff9e4665449b7c7d0e4238cbda664b0dc9559c6d2a203f06deac9").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b8a0c05a3d9eb4f514367f95c64f40e57b072dad7f32db3bc39ab4f6df094379f32fa0414e80b353f2771a27d62ce85b43efa5571c8b8883b00777c301aa4359268266").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b8a0abaefaea1c4a87f79e0f7630660fe528e3a6724e6a8ffe1cf0281623e005733da07ba9491c1efa7d9700a45ca227d6d460bd286ea613d14978f6d269a711f3b92a").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a04d28ef4425a1c5d6753bfca4d8361c5a1cab65ab37bf3bb08d29f4c589a88385a03c318c329479bc443e2f502218ca8e420fa94863b4cd5e90b43204a089acd460").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228e041603c110808328d3b7a0b0c7ae730e44676e3eab4a6ea754d8c1ae3141c9af2de2872681504b3daba14fa019f10a927fceb6f6baf9732a1fd9fd5ac2063b39d793a0dc1c35ffd8d20d54c2").to_vec(),
                            hex!("f86d01848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147872347da015bc110808328d3b8a017c752cb24029dea0f5ac3354ac83f6e57cda4cdb84701be47a78adf4f58c268a042cc51866a5b654e2cf864c1727911c690cfd63a8f3b1d6c21282ff2907b5d63").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a0d0962628f6bbb0dbf59863fe6640c28184813b90550a3d80bbb97254e2548316a07eeae9c68c82e246c52363250f907a2e2b685712c74659f3b83511c13d5d6eec").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b8a0391ada7e9f212760f85c2abaf1e1b0c92dd7f5a9592f87bc1356d335c056ed7ca05b0d28a57f8bc3945b4aba8f97d9eff856453df943513f9bbbebea85dfa96be3").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b8a026dbed6f6c65e82c54243b5a6f3b7898efe3bcaea26b647f5686588aebdef22ba0664632964e514260f19bf39e8c72af8b541c9c923d770a7bf25e8e1638e1f642").to_vec(),
                            hex!("f86d01848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147872347da015bc110808328d3b7a00c970e13a41f942a0807e34c019933ba208d74add91dc23874dea5f66e1646d5a05d0f8036c0993d26d221ea2b9dd7d0790cc95056e0d9068cfa69f281faa48233").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228e041603c110808328d3b7a0c79e0295945db7ffdaecff8328aa1cb1325cc1a4d82361e7d8b6336965205f97a0317e00f115a4db52154728b4eb9e13dc643bcca492cd41f30a2ce328c327b2ed").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a05d007180c60a0577c4e3f1896855f3b6d42291b68f80c5542e89251766185e78a07dc49220baedc5b4faa87ef65627cb8ebf0a284788fefe7008e79a80175951ce").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478908b2228e041603c110808328d3b8a0094da0217ebde0c78f2367fec59569387524ae15872a37a396556a813c91cefaa03f3f759e5d2b1c4f431980788a018537b7e7129b5c3aff63665f5bb7c5dd6e91").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a03882307e5ac4524f13d3ddb99daba53ae77b17a0cda7fb367e812036fa820225a02ce26ab9b8a3995ed1b03ffcb96ce61e9b4b5c796271c013df45f9058a10e316").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b8a02b995e24517a0faf9efcdc800359c2c22413dfd0b1e7a42f5c374efb56caed3fa05a3165c73185c8ee205675083c9524c13c3faca209260dbb94ece4067e684fe6").to_vec(),
                            hex!("f86d01848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147872347da015bc110808328d3b7a08240c8cf3c1532cc099ca154538e6a97cc8abd633d1d54133c32c9ba81e89b80a04d1b898d661679613301c9e0a95842c6b0d32f260895d757eeba7ea15bb4d73f").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b8a0d7b46c71d30d020f1ac3ffe3a58fdcedaae2a3fbc588fcf0b84b43d7baab05b4a04fa7a0e9139b0a92dd0cbe62708d36c52c6318faeee77122ef87e75b1ebf4c5e").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b8a05ac8fa5d5abcf53e2099389e3c9eeca50ef2e4928ee4f87ad27f67d5e182b171a003bfb52ec175f6dea9ba49cf9f2d58d25f83ab131a458dccc1d207bf13ca0886").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b7a0205ff69e5617e773851b84a638f3d41a046d8b01879db86c33788d16042e0085a011ec4863d9d4e717f11a00923d9287dbd0beeaccc5780f8f09e0e78c0a7163fb").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a02799b22ba4bce7ff6331ded7af1f4061f25b353aba628b56c1671accb22d8aa3a02020c5c5cb1bda5e5708e424e63fbe953ef72434c60300707e53ad7160de2160").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b8a061393713a0510724eb2860cc55e19f362860eba6e58a86cb6460f50182a586a2a01a0fa10535f600e0a7354372529e14be5e23b65ab24ccd0d18614bc9434bb2c7").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c314789037a8963ead66bc110808328d3b7a0e4eda568114a689a1f3f05baa47b022853eab6a594f14588f1e359a322968fe0a014b17776d7b25d7dd54be6188b3075ea34f44cad49b8d5b8f9a725b534ebfb92").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b7a0de98d4260df915490a8247dc1b280ed899672b20ab0bb32e71ff90d2fb8f020fa045ecf0dc6eee1fe1774ad6873c988f12950a49efce8621c94d6cafd316fc3ff6").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a0282dd5f1a9a5f3dea162b006825115acefe2419dadd8fda961e675d541988f32a02c1c603eff2b46dcae582fea67f4f3c8a46e5d1b27685e68f5b38b4cc6534b8b").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b7a071e5edf624530646cdfe4464350bf5d7533f2b9306e53f284cf9cac09fac073ba04484f7363b3f41cef9c8fda4b010a3f1f25456f40e85f2401753bb79f060251f").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b7a0f1ed8070ea47e0e5ae5bf4ce33d11ecfeffa1216ee92c6e77a34767002a48154a066727a0a6b0d16125d415d4263ee38678f825eb6ed8707a1e0f473d96fe810fc").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b8a0e5b2cba2d85c83c4ce897c8062d2d96be8148f45c76f6b088787ddc62ec47638a07cec9a53df648d05486437c3a7f0fefafa1b0721f256af3fae3f90273116d92e").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a037f855a2a228a15e90cb8a6c6dec5946552f7eb97dd826c4c2cd59e6bed076faa01462fef47e8a5a44bdf2be028024b9af224dd38cd716d00885c9688db9c80764").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b8a0f68a0cc57b0b06eda4d0908f9c7433d84940fead0870440b3d711d3304eb4b95a022a0da66f2aadce14e98fd3ef9d8a1d8106ec839029cffee202a7fd200cae0f1").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc71f340f3c110808328d3b7a0330448f06133aace2ff68ff8cd71d29785cf97a2fa69d2ed5b7d26c6e53239f8a076e5b0c5207fd7e40ac2c22a834c51c16cc7dc600f9581eabd864b45ddfb7172").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b8a0aec3c91de2552e686cb9138ababbb6a69b736ec8c39d7736da3d605b179b22dea07823974ceabc803b95dfa53c465d832fd283230560c636ed7a7e2c0bc367553f").to_vec(),
                            hex!("f86f02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b8a036dbe9a906c2abcafabb97fd5f6c095bf4de2102354cbb7ea317385159fad285a05815835ef7d38499900f69639b05c987a826e78c225acaeb69312059f5ed5d23").to_vec(),
                            hex!("f86e02848357395782520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef7ffbab7bc110808328d3b8a06ae4347298db07f30c463b545178d50e8198adbce5be3aec3114d233157bfba19f3f96e6a414c8bc4c99599ea88ad9d838a96fe6a56e13ef45290a8cefa72532").to_vec(),
                            hex!("f86d01848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147872347da015bc110808328d3b8a01d6aaae43c313e3038e808736d58c40ef4805fc31c2ef891cea066e7fe0e4a0ba00e0840baf631bf3ef40d2c0cc8f5df13f2d88be70384958f96d55ef1c54e1aa7").to_vec(),
                            hex!("f86d01848357395782520894f6c52944390a63a2b31ceadbec5b0424026c3147872347da015bc110808328d3b8a0052cac7da42e630627b959d59abefadac93bdac9a1acfa93e01ab979630e44aba0022475e93a2b1062574e54f6af6848bcadd9da2312ce043480a20a54c97bbb96").to_vec(),
                            hex!("f86f02847fc3ad7682520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc7318a05e3218808328d3b7a0985394d2be3375bd017cdf02a9e9d84a67ed30cbcafa08891705972562e9cb5ba07984680b4fc5c71c680e70a91a32dad4c615c82e9f8e84a80a85b8646c977cd3").to_vec(),
                            hex!("f86f02847fc3ad7682520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc7318a05e3218808328d3b8a07a897a7055609f4e144d20b8d669b58c98f251a21b7f0d032bb0fa2d9c53466aa07f4bea3efe02f7b23a89226e1c984f3447f7b93a37ca4a1889939a568bc27f09").to_vec(),
                            hex!("f86f02847fc3ad7682520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc7318a05e3218808328d3b8a09350f6580fa0c3a05a3719be8f9bc573975d3df13d89c31a1b657ed7289b165aa03bedbcb53936cae3ea9d9362d26512a64485ae1aedd66634a74456de483e41fe").to_vec(),
                            hex!("f86f02847fc3ad7682520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef81210ae63218808328d3b8a054e11c42343637ee603417e0512e461ebce97f81c17711c8841f299b74d10581a03c8740ef8906dd13a7cfcf2ce5ad3966eecc3281b24daf3c54115801280df002").to_vec(),
                            hex!("f86f02847fc3ad7682520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc7318a05e3218808328d3b7a0a2611ce79d27804990116c4cfeb590d7562c99793c3f907e30313088d6ce093ea0606d4ba29a63aa1d7dc6e59fa27d4a942f884c1202ca00d53ff8d9cf4ebbc77c").to_vec(),
                            hex!("f86f02847fc3ad7682520894f6c52944390a63a2b31ceadbec5b0424026c3147890537bc7318a05e3218808328d3b8a0fa8cac9f239225022756d427f9842417ca6890acac70aad436603c6c87922649a00ccc34510bf00b894a05894e80216b6c71f25c1dd7ea208b76fe684a4107096f").to_vec(),
                            hex!("f86f02847fc3ad7682520894f6c52944390a63a2b31ceadbec5b0424026c31478906f4ef81210ae63218808328d3b7a051de19858bc59f4dd4962148ed6da068580d308e4367bafbfc5197a520e23329a053df12f742b8e04a93f470491dd8a2a1f0e71be6ea4183113d867bb292f66091").to_vec(),
                            hex!("02f878831469ca821c1884054c12c484500a7ae98252089422007a12f6494eb73165bdc5a475771ef225532589022b1c8c1227a0000080c001a05ff8b522c9978d15dccc4ffe7098fea10155b972b62cdd276aa5ad57f7ff847da04d58c5b6762ccd1bc8686cf7bf8639ae16029b8484308f180666ee5b8d905462").to_vec(),
                            hex!("02f879831469ca8301455a84054c12c48452be174282520894f1c26981dd8fd214fe9264a897d9e6cda96db6488901bd330e086a88000080c080a02e4b11c32c77228914681b831d73b31b185e3adc5726948b4665d612ab11e101a00d673e8ea187682e58a282483ac13409b0ddaebaeb2a1e537a2020773c707b0d").to_vec(),
                            hex!("02f879831469ca8301455b84054c12c48452be1742825208944d170baa29a81df2877aa0a3fd798d4cbb92f1e88901bd330e086a88000080c080a026fe1e593474a51f03ec7ad785060b8d01e3b1e796b449728ba51500f8b6364ea0319ce27699538560d85b677594439af6bac145947e0f146c42d7ec35afa9f889").to_vec(),
                            hex!("02f879831469ca8301455c84054c12c48452be174282520894be7269d183e23e093bf295546a63bf958db8f65f8901bd330e086a88000080c080a07936bb27b9360704e471ce5a024ffe004dae65fa90689b695ed36fccf3edd2c5a0635c9d1ec6e31ac0f5a3c7ef801dacd3ec7d45dc9e3f53ba8b05fa9523b7df88").to_vec(),
                            hex!("02f879831469ca8301455d84054c12c48452be174282520894f9a13d1ea489df1f1a8915436ad3f4930c6d16868901bd330e086a88000080c001a015889fd79b0e0957aa7655d4f5d0006d9c722784f70ab1b182371ab3293283e7a03225f18a36cd771217015bd7d229d22fc6794c2e56a83e82453f977cd6f6ba6e").to_vec(),
                            hex!("02f879831469ca8301455e84054c12c48452be174282520894b00f3c318685b1e6bebecabd06578dc00fa1dd738901bd330e086a88000080c080a04496ea4986bda6bb05fba0bac03fb879355235d4023b209de472598a400a749ba032e8c8a47fefbc9012d3ee22f969a0d2c12895dd26747b7acffda516d6022f76").to_vec(),
                            hex!("02f879831469ca8301455f84054c12c48452be174282520894b0e54b7088acadbbf4eb35f0db7e35b08bce340a8901bd330e086a88000080c080a001c2b54e13eeff72eb1fdf57d8247c7cf2861330ee719b201ab566c756124f16a0739cef53e7f802a37f4d582715271f7f7069185c12cd24f691b4906456797b96").to_vec(),
                            hex!("02f878831469ca821c1984054c12c48452be174282520894501f11a1152bdd74048dd1dfaeb73dff0b1c893689022b1c8c1227a0000080c080a063819ef57846d279d4f993c38cadb26ac6df6333128b9888f1955c7326c9bddfa006293ecc1dca3fdb4b30c44b1a92f7e375665337fbbd08549c18a829f425c9bb").to_vec(),
                            hex!("02f878831469ca821c1a84054c12c48452be174282520894411a1b1909deee4b60a5bc16d6c2dc35bf0b77a589022b1c8c1227a0000080c001a00dcbe8fa6b73e98875d345a6e4fc7d90bf91d3333c4339ea0347767d17f178dfa072e6eb8af05084d4d09d39d4fa2c4d0fd5ac5d56f3120f332acd380815430e39").to_vec(),
                            hex!("02f878831469ca821c1b84054c12c48452be174282520894f8da77605dabf5e0682e8eed9020fa1d90ddd91689022b1c8c1227a0000080c080a082e8ee47eb1f5e41b372341681fd05aef326f8309d98f855ec96b5e108c1cc67a033079cd65164450a85da755525d6395434cd4d96a27ffcac68fd1b6d7bc49262").to_vec(),
                            hex!("02f878831469ca821c1c84054c12c48452be174282520894adb12988cae14f9170b6f38819736636a1fd129089022b1c8c1227a0000080c001a042ec31c90dc676bb0c01dd0082ca931ff3db486e2e9ce31b5da915a47a46e326a03f766a9e78805d3d4731f2fde3dfbefb74b49373f8150b731427931a083f9312").to_vec(),
                            hex!("02f878831469ca821c1d84054c12c484490fd6b282520894b731c7d4947c5cc24984f87814878cf0e131117c89022b1c8c1227a0000080c001a01fed2cba97abd44b8236c786d94f273b50c672e5ff77ff397df708283ed90e26a0370872b3f3ad0694053dc8af16a4fa3e39bf482c0ff4937366023c8a9a3d2969").to_vec(),
                        ], 
                    },
                },
            }
        );

        assert_ok!(&hash_root);
        // assert_eq!(
        //    hash_root.unwrap(),
        //    hex!("b9eb2caf2d691b183c2d57f322afe505c078cd08101324f61c3641714789a54e")
        //);/ fix
    }

    #[test]
    pub fn test_hash_block() {
        let hash_root = merklization::hash_tree_root_beacon_block(
            BeaconBlock {
                slot: 484120,
                proposer_index: 52624,
                parent_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                state_root: hex!("cad5f72126b7e026f799465886a9dda0fb50b1baff4c6f733e2a16c500e4a2a7").into(),
                body: Body{
                    randao_reveal: hex!("82c58d251044ab938b84747524e9b5ecbf6f71f6f1ac10a834806d033bbc49ecd2391072f9bbb4758a960342f8ee03930dc8195f15649c654a56767632230fe3d196f6499d94cd239ba964fe21d7e4715127a385ee018d405719428178172188").to_vec(),
                    eth1_data: Eth1Data{
                        deposit_root: hex!("d70a234731285c6804c2a4f56711ddb8c82c99740f207854891028af34e27e5e").into(),
                        deposit_count: 0,
                        block_hash: hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
                    },
                    graffiti: hex!("707279736d2d6765746800000000000000000000000000000000000000000000").into(),
                    proposer_slashings: vec![],
                    attester_slashings: vec![],
                    attestations: vec![
                        Attestation{ 
                            aggregation_bits: hex!("ffcffeff7ffffffffefbf7ffffffdff73e").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 0,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("af8e57aadf092443bd6675927ca84875419233fb7a5eb3ae626621d3339fe738b00af4a0edcc55efbe1198a815600784074388d366c4add789aa6126bb1ec5ed63ad8d8f22b5f158ae4c25d46b08d46d1188f7ed7e8f99d96ff6c3c69a240c18").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("edbfedffbfffff7dffaefdbf77d3ff7e37").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 25,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("b8b4efa0b212bc0e98a70837a0f2e2a548ed2bdc493ff9ce83d8ce9290d6aec93ad93ea01ea5f7946c8cb8a5ac01981d197a8028cf5a58656fdeb3c3572368dba695b4686aff04a4e72db88c666871defc43c61b89dab3e5b675db131839f172").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("effffff7bfdffffffffffbfffeffffdf1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 24,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("b3c8813cc0bb32bda17914e32d1c76dc2ff4a304ac07b35a6636d9b77d4f10062c83aa41ee5f6d5622934512e655deeb02d3830c44b6267a5e0dfabff3eeffe9d02229edcac2a345546c3cecd62e97013e1c54996be344191727b70bcb9541eb").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffededfffffffffb7fff9fdfffffffbd3f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 21,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("ac7c7eaaab73e4566bf8d826079efc846c40a309aba7db8e6de52173c57eb4c2c2bcf79c7ea223d3ce230afa837b0fa109c36448737664c908817d27538a863a3ca652103d6fe6d99213214697a4c987d17cb283d7413acd7f711b914878c6b3").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffffff7e7efffdfdffeffffffffdfdfd1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 20,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("a574395a9208a5b543775ded57b8ed012ffa0e61c4380879de166aff1eac506f661df540140384da08e751099426569e00306059aa6ca04a5e5b89c63c78464cc4a6edd38e74d19ab73f52cbb6718a0aec81ee7d1e16ed9ebf3495b0f0956ef0").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffffffefddefffebfffdffffdfffbfef3d").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 11,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("a9eff122acd4991b82ed1a60676b373423be7d0780f0905aca5ef72de8e02ed473888c26db52d6f74ac0d60d9d652dd812f1b33fdc867137640bc9927a481ab89a3ce165d54fa9f574fd862fcf5d35d4515082ca990f66e4bbef0bb4414cb12d").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("dfdbfffbf7fffff7fffe6ffffdffbdff1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 15,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("978d294b649bd0f71f25ab1192158f866d5af0b14dffdd262c4616df7be5fb608d996ecfe2d525db754c681e6f10125c0cb2d25b1d49269b1389e59ea79760342f6d2690b85f00c1513e2674c5da10cd10e59fdf0c3da444ad26d7c654c44d3b").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffffaefffbffffefffffffd63fbff7ff3e").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 23,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("a39f32ed3e02424ededb0174f170b29690eb23160f4d42f4e154185728cc1d2705c487b6c9be6687ab70a690530d8d9816c6c0f1bfd15a60f518a569bb8a95dd2cb57cdcda7039a7f297e13010388159f74f6138236d51b0aaeefd93a189bf42").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffeffffffefffdddfffff77fffffdeb23f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 2,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("822ec536c4994e864a29234a6cd6721b65c16bf54df8c3bb7a1a9a7e5cdae58d3a035521c533f80811c67b0e98ae66671396c246aa3f7011db2588a09a61c782126ba2d03ffa38aea5b4682395af54ce7dff91ae03fcbe77da6edd13843e2c10").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("cfffdffffd7fdffff7ff9ffffbfdffb53f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 7,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("9488ddbfeee168d54c6693d895aea392af26005a9cb108f7f4697f031f8f0a68b50d394031869ef5c14d850022afa0660eec6238c71051a12c9b1d19934dd7f75e178f684e8eb72cd7f8403a28200630d96e84be679b93ba5c912ffc0614b0f9").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("effcfffffdff7bcffffffff7fbfddfef1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 6,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("881710de911d66e0b1525b7b78afff36615a61c110d9fd3417bd6379823599cd6d3db3f46c2c6d89cb8672a80c9975bc0a18a2d2c54f1b3d9f1b078d541bec80f615a00b1a5676a4750caa0a6d37300e11b3dc0f912a20575eeff302af8bf4dc").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("feddfdddf7f7ffffffd7fffffbf3fffd3f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 16,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("a9db8a4fb7236700ba563dd798244397d8a95a4bda209291a57e99c45c95c38471a7c24106cf8b9eb9a22da4ef0556f51270c305ce4008d623f2fa5614ed329d86ad54c562ca1338561f20474aef251d9d870ba44f93aadffb6f497ba46b61f1").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffffffdfebfff7dfefdfffadbfffbfcf1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 13,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("b7922f4f45c3cc1b92823f5d693f2cda800b2b3a068766aa96d871a7b2a3449c9e9f3ad7ec3651ac09c6fd705f889cd40f976d9d0fb1dc3b87f072da7dcd8a7e78e7ff23ff21582e93b59ff82c513dfa829e88ec535b1507c5eb4a778e8d2243").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("fb7b5ffff6f6edbeffffefffffffffff1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 19,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("a6ac3658da203ef274617be2beef05dc615e3321fbe2a3aa563daec37035400eea8b09536561632809cf92a19a99d7580f9fe22499f3ebfaf1ac1099db4f8a45b7cf1a4bf381a6056f389d48556a387e0fd9188a8ecc29f329e6a9cff966982a").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffde6fffffffeffffdffe7dfc7e5ffff3f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 14,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("93f13bcbff3a3dadde7191ba614beb0a3c4d186ea4a13df4aca2b537393ffebd265fbd3ce72ec3f143e03bfbdf8d8b6412accff9b127befca13b7b521434e1e2823fa90134767a2d8bee7f2af6bd5b5c7aa5ac24596ecebda19561affa29f81d").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("fdfffffbfff7fddfddffdffbbfffbffa1e").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 8,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("b3f53b7e1e81bd4be2ac612e79113e4e72dcf426d36a7554a04e06626e8a46c2a7108f94c4af818e82e4d47a4e04d7bb107250964d725aec488dc914c47e3a22312ee6194bfde8b5d6d87b59acafab4eca8e8b8b10be764321fe2618a75a1384").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("cffeeaffeffeffffb7fffffe3f7fefff2f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 9,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("84f2b0c785c136813a08992652f28578e1724e8ed1d6ed4cd35056077b8c536f9c5e413c046e8f332104c08f6a4cf1a703922d02f8533bc7d6a118faba44c219229437cf0dde08ea13c9ff2540d5c6bcc3aceb4402b94745c0ad26d1931e6f1f").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("bebff7ffd7f3effffaffffbffeffff7f17").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 1,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("96182aae3439804af5779f3267121233102fb99fc69aa3d78e22c397ccf6ed86408ff21c69197f132d61caea74c7c3d6091debaae929c15c5277bb02301e6c02f5aef55796e678bb3175205707da4d98c335e71d2127c5e029e0f62d74e5ad2d").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("bfffffafd57fdfff77ff3fffffd7ffdf1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 3,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("aeeaab0745087b0a95bbeeefe011393acff13305cc799f459cd5895294a45e3008377a3cd2582f253986e7db0521100618a99acfa7201897f579f6b15c9c6c8f266421aa033e26e4bafeb6d6bcb2395fa7c349752f2a26f3d297372b8e42fe1a").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffffd77fd7bfdf5adfffffbff7fff7ff2f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 4,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("a5053b79aacb223f0bc51189d96e39d7fa9a08fb42b8dcedc8747141ddd2bb7325367f43c41fe633175b208042a6a6800a27c6ae0a8e1e1ac898a9578bc209a8b816f6e95d9914742ce3f53f42a90df48fc5f226d09e1222f560d6c11dc42427").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("6ebffffeffffeffbfffdfef47f7ff7fe1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 17,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("91c9d68fd5859a42bb620aa2c9499c78f79c2e3c7d0d61cdf1efb47d3c9422d60f700d5c3f546cd431eeb08b933b98680603c1e300d4f7dccd8d58dd12fb9dd48664f4cafb1b3c7b943ce87fca4921832f015932123c1dc1641e96af78d11d63").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ff7ffeef76faeff7ffffdfdbfdffffec3f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 18,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("88b6ff5bd0d3b06931b484de72da01ab3670e6a2c65be2112ea83ac868a791c401dd92494aa504fcb1838c7b63b5f79c06a8274f9d357e451d34f32ec1edc79bb38ef3b96527ead0f7c50b4e0adb062ec887d1633656254706fb285d846cb00a").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("7ffbfffffdccfff7ffeeddffebfffff61e").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 22,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("916083657bcc9f5f9fdfd009e9c65461f214570c9829495e03cd568cd40e1341598ea4ad53147a8ec6a39cbf8858381e09070371bc3fc0373053a14409db6a340f19e5c5528f470c974d3dadba251b7a2e6225670b4f0241b157dd2ce7325a15").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("ffffdfbfdffdcff7feebfffbeaffff2f1f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 5,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("a259e1026193735d542ec4670f1250474ac6072f6100d02ecefb063227ebfbe4f7f8ff8ffafaf50774cccf4c5a84a3be06b06c579953a2a95cce9b1d43fecfadc0a2a0bd0815375dcb69f9017a1124073cd06cae5ebb1f36c68fadc25ce13f7c").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("fbff9ffdff3ddfffbfbfb7ffafd7dff61f").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 12,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("9133cd7e63fa2d7df3f7d99754201c3a44a51ff76bfbdf521640611be6ad68d720db7c6173ec599cb3503da45ca8a25304f866c7e4bf5b24725536a50244dda108c3ae75dc7a18d1c486c3cef23a5fd05dfb357c5f7deb94ec553c4c69a30648").to_vec(),
                    },
                        Attestation{ 
                            aggregation_bits: hex!("feff9afdd7bfdaf7dffffbfbfedfbfff19").to_vec(), 
                            data: AttestationData{
                                slot: 484119,
                                index: 10,
                                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                                source: Checkpoint{
                                    epoch: 15127,
                                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                                },
                                target: Checkpoint{
                                    epoch: 15128,
                                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                                }
                            },
                            signature: hex!("87b3d569284d0ddc400f57bcfba4a6ae48456a31470b8f43fd0008e3d4cd8dc4e9acfccc5ef569cddc8282d7d1890700091382632ab1c45e85c55a661e1bcb1b905ff6f6fad2e4ee3c4aa27fa371d40a0799a3df717eabf79ef17c133ce9d040").to_vec()
                    }
                    ],
                    deposits: vec![],
                    voluntary_exits:  vec![],
                    sync_aggregate: SyncAggregate{
                        sync_committee_bits: hex!("cefffffefffffff767fffbedffffeffffeeffdffffdebffffff7f7dbdf7fffdffffbffcfffdff79dfffbbfefff2ffffff7ddeff7ffffc98ff7fbfffffffffff7").to_vec(),
                        sync_committee_signature: hex!("8af1a8577bba419fe054ee49b16ed28e081dda6d3ba41651634685e890992a0b675e20f8d9f2ec137fe9eb50e838aa6117f9f5410e2e1024c4b4f0e098e55144843ce90b7acde52fe7b94f2a1037342c951dc59f501c92acf7ed944cb6d2b5f7").to_vec(),
                    },
                    execution_payload: ExecutionPayload{
                        parent_hash: hex!("eadee5ab098dde64e9fd02ae5858064bad67064070679625b09f8d82dec183f7").into(),
                        fee_recipient: hex!("f97e180c050e5ab072211ad2c213eb5aee4df134").to_vec(),
                        state_root: hex!("564fa064c2a324c2b5978d7fdfc5d4224d4f421a45388af1ed405a399c845dff").into(),
                        receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
                        logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").to_vec(),
                        prev_randao: hex!("6bf538bdfbdf1c96ff528726a40658a91d0bda0f1351448c4c4f3604db2a0ccf").into(),
                        block_number: 477434,
                        gas_limit: 8154925,
                        gas_used: 0,
                        timestamp: 1652816940,
                        extra_data: vec![], 
                        base_fee_per_gas: 7,
                        block_hash: hex!("cd8df91b4503adb8f2f1c7a4f60e07a1f1a2cbdfa2a95bceba581f3ff65c1968").into(),
                        transactions: vec![ ], 
                    },
                },
            }
        ).into();

    
        assert_ok!(&hash_root);

        let hash_tree_hex: H256 = hash_root.unwrap().into();

        assert_eq!(
            hash_tree_hex,
            hex!("f7c410dbb50104bd95a66ad2b564cc18bbcce06391e572b61968a2d78c3ee4cc").into()
        );
    }

    #[test]
    pub fn test_hash_tree_root_execution_payload() {
        let hash_root = merklization::hash_tree_root_execution_payload(
            ExecutionPayload{
                parent_hash: hex!("eadee5ab098dde64e9fd02ae5858064bad67064070679625b09f8d82dec183f7").into(),
                fee_recipient: hex!("f97e180c050e5ab072211ad2c213eb5aee4df134").to_vec(),
                state_root: hex!("564fa064c2a324c2b5978d7fdfc5d4224d4f421a45388af1ed405a399c845dff").into(),
                receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
                logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").to_vec(),
                prev_randao: hex!("6bf538bdfbdf1c96ff528726a40658a91d0bda0f1351448c4c4f3604db2a0ccf").into(),
                block_number: 477434,
                gas_limit: 8154925,
                gas_used: 0,
                timestamp: 1652816940,
                extra_data: vec![], 
                base_fee_per_gas: 7,
                block_hash: hex!("cd8df91b4503adb8f2f1c7a4f60e07a1f1a2cbdfa2a95bceba581f3ff65c1968").into(),
                transactions: vec![], 
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("4c74e6119faeee22c04ef02fb6d8db26799753e2a9efcde6ea60cbac1f38cfd2")
        );
    }

    #[test]
    pub fn test_hash_tree_root_attestation() {
        let hash_root = merklization::hash_tree_root_attestation(
            Attestation{ 
                aggregation_bits: hex!("ffcffeff7ffffffffefbf7ffffffdff73e").to_vec(), 
                data: AttestationData{
                    slot: 484119,
                    index: 0,
                    beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                    source: Checkpoint{
                        epoch: 15127,
                        root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                    },
                    target: Checkpoint{
                        epoch: 15128,
                        root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                    }
                },
                signature: hex!("af8e57aadf092443bd6675927ca84875419233fb7a5eb3ae626621d3339fe738b00af4a0edcc55efbe1198a815600784074388d366c4add789aa6126bb1ec5ed63ad8d8f22b5f158ae4c25d46b08d46d1188f7ed7e8f99d96ff6c3c69a240c18").to_vec(),
            },
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("a60acb46465c9eda6047e2cc18b3d509b7610efcbc7a02d28aea3ffa67e89f5a")
        );
    }

    #[test]
    pub fn test_hash_tree_root_attestation_data() {
        let hash_root = merklization::hash_tree_root_attestation_data(
            AttestationData{
                slot: 484119,
                index: 25,
                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                source: Checkpoint{
                    epoch: 15127,
                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                },
                target: Checkpoint{
                    epoch: 15128,
                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                }
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("351d24efe677a40e3b687f8c95821158c3a3bb7c41c43b51187d4c1df690c849")
        );
    }

    #[test]
    pub fn test_hash_tree_root_checkpoint() {
        let hash_root = merklization::hash_tree_root_checkpoint(
            Checkpoint{
                epoch: 15127,
                root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("c83bfcaa363a349b6869d70dcfe430f6199f8da7b01eb92d05a0860efe19dcec")
        );
    }

    #[test]
    pub fn test_hash_tree_root_checkpoint_target() {
        let hash_root = merklization::hash_tree_root_checkpoint(
            Checkpoint{
                epoch: 15128,
                root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
            }
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("8e9806a6b5ce2b73fb0d48c6ef36c658d9960902086f526d8932db84ceec1101")
        );
    }

    const SYNC_COMMITTEE_SIZE: usize = 512;

    #[test]
    pub fn test_sync_committee_bits() {
        let sync_committee_hex = hex!("32f767f59f4da9dfecae0defcbf76776ffe1beaf3e6fb1ef4be53bf613f9159fe95db7abf1e388fe4fabffcf28fddf913fd742ff5f55bb51bdcf6b6fdab3fbc8").to_vec();

        let bools = merklization::convert_to_binary_bool(sync_committee_hex);

        let bits = Bitvector::<SYNC_COMMITTEE_SIZE>::from_iter(bools);

        let hash_root = merklization::hash_tree_root(bits);

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("29247bcfa74b74783bab210386e07785c25968a136a7a899c3e48ec79d0eef2a")
        );
    }

    #[test]
    pub fn test_bls_signature() {
        let signature = hex!("af8e57aadf092443bd6675927ca84875419233fb7a5eb3ae626621d3339fe738b00af4a0edcc55efbe1198a815600784074388d366c4add789aa6126bb1ec5ed63ad8d8f22b5f158ae4c25d46b08d46d1188f7ed7e8f99d96ff6c3c69a240c18").to_vec();

        let signature_conv = Vector::<u8, 96>::from_iter(signature);

        let hash_root = merklization::hash_tree_root(signature_conv);

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("f3a7f166009ec32adb8848865fc113a4a1609ff228849075be974b52ec94ed84")
        );
    }

    #[test]
    pub fn test_aggregation_bits() {
        let agg_bits_hex = hex!("ffcffeff7ffffffffefbf7ffffffdff73e").to_vec();

        let bools = merklization::convert_to_binary_bool(agg_bits_hex);

        let bits = Bitvector::<SYNC_COMMITTEE_SIZE>::from_iter(bools);

        let hash_root = merklization::hash_tree_root(bits);

        assert_ok!(&hash_root);

        let val: H256 = hash_root.unwrap().into();

        assert_eq!(
            val,
            hex!("ac4175b816fda9a6bc2a59c905a9df02383763176b58c3cd2823a53c107ff3cf").into()
        );
    }

    #[test]
    pub fn test_aggregation_with_hex() {
        let attestation = Attestation{ 
            aggregation_bits: hex!("ffcffeff7ffffffffefbf7ffffffdff73e").to_vec(), 
            data: AttestationData{
                slot: 484119,
                index: 0,
                beacon_block_root: hex!("2e93202be9ab790aea3d84ae1313a6daaf115c7de54a05038fba715be67b06d5").into(),
                source: Checkpoint{
                    epoch: 15127,
                    root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
                },
                target: Checkpoint{
                    epoch: 15128,
                    root: hex!("3a667c20c78352228169181f19757c774ca93d81047a6c121a0e88b2c385c7f7").into(),
                }
            },
            signature: hex!("af8e57aadf092443bd6675927ca84875419233fb7a5eb3ae626621d3339fe738b00af4a0edcc55efbe1198a815600784074388d366c4add789aa6126bb1ec5ed63ad8d8f22b5f158ae4c25d46b08d46d1188f7ed7e8f99d96ff6c3c69a240c18").to_vec(),
        };

        let signature = Vector::<u8, 96>::from_iter(attestation.signature.clone());

        let agg_hex: H256 = hex!("ac4175b816fda9a6bc2a59c905a9df02383763176b58c3cd2823a53c107ff3cf").into();

        let conv_attestor_attestation = SSZAttestationTest{
            aggregation_bits: agg_hex.as_bytes().try_into().unwrap(),
            data: SSZAttestationData{
                slot: attestation.data.slot,
                index: attestation.data.index,
                beacon_block_root: attestation.data.beacon_block_root.as_bytes().try_into().unwrap(),
                source: SSZCheckpoint{
                    epoch: attestation.data.source.epoch,
                    root: attestation.data.source.root.as_bytes().try_into().unwrap(),
                },
                target: SSZCheckpoint{
                    epoch: attestation.data.target.epoch,
                    root: attestation.data.target.root.as_bytes().try_into().unwrap(),
                },
            },
            signature: signature,
        };

        let hash_root = merklization::hash_tree_root(conv_attestor_attestation);

        assert_ok!(&hash_root);

        let val: H256 = hash_root.unwrap().into();

        assert_eq!(
            val,
            hex!("a60acb46465c9eda6047e2cc18b3d509b7610efcbc7a02d28aea3ffa67e89f5a").into()
        );
    }


    #[test]
    pub fn test_aggregation_bits_hex() {
        let aggregation_bits = hex!("ffcffeff7ffffffffefbf7ffffffdff73e").to_vec();

        let bits = Bitlist::<2048>::deserialize(&aggregation_bits).unwrap();

        let hash_root = merklization::hash_tree_root(bits);

        assert_ok!(&hash_root);

        let val: H256 = hash_root.unwrap().into();

        assert_eq!(
            val,
            hex!("ac4175b816fda9a6bc2a59c905a9df02383763176b58c3cd2823a53c107ff3cf").into()
        );
    }
}