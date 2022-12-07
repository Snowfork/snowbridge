use frame_support::BoundedVec;
use frame_support::traits::Get;
use ssz_rs::{Deserialize, SimpleSerialize as SimpleSerializeTrait, Bitlist, Bitvector, U256, prelude::Vector, prelude::List, DeserializeError};
use sp_std::{prelude::*, convert::TryInto, iter::FromIterator};
use byte_slice_cast::AsByteSlice;
use snowbridge_beacon_primitives::{SyncAggregate, Attestation, Checkpoint, Eth1Data, BeaconHeader, AttesterSlashing, ExecutionPayload, SigningData, ForkData, SyncCommittee, AttestationData, Body, ProposerSlashing, Deposit, VoluntaryExit};
use crate::ssz::*;
use crate::config as config;

#[derive(Debug, PartialEq)]
pub enum MerkleizationError {
    HashTreeRootError,
    HashTreeRootInvalidBytes,
    InvalidLength,
    InputTooShort,
    ExtraInput,
    InvalidInput
}

pub fn get_ssz_beacon_block_body<
	FeeRecipientSize: Get<u32>, 
	LogsBloomSize: Get<u32>, 
	ExtraDataSize: Get<u32>, 
	DepositDataSize: Get<u32>, 
	PublicKeySize: Get<u32>, 
	SignatureSize: Get<u32>, 
	ProofSize: Get<u32>, 
	ProposerSlashingSize: Get<u32>, 
	AttesterSlashingSize: Get<u32>, 
	VoluntaryExitSize: Get<u32>,
	AttestationSize: Get<u32>,
	AggregationBitsSize: Get<u32>,
	ValidatorCommitteeSize: Get<u32>>
    (body: Body<FeeRecipientSize, 
	LogsBloomSize, 
	ExtraDataSize, 
	DepositDataSize, 
	PublicKeySize, 
	SignatureSize, 
	ProofSize, 
	ProposerSlashingSize, 
	AttesterSlashingSize, 
	VoluntaryExitSize,
	AttestationSize,
	AggregationBitsSize,
	ValidatorCommitteeSize>) -> Result<SSZBeaconBlockBody, MerkleizationError> {
    Ok(SSZBeaconBlockBody{
        randao_reveal: Vector::<u8, 96>::from_iter(body.randao_reveal),
        eth1_data: get_ssz_eth1_data(body.eth1_data)?,
        graffiti: body.graffiti.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        proposer_slashings: get_ssz_proposer_slashings(body.proposer_slashings)?,
        attester_slashings: get_ssz_attester_slashings(body.attester_slashings)?,
        attestations: get_ssz_attestations(body.attestations)?,
        deposits: get_ssz_deposits(body.deposits)?,
        voluntary_exits: get_ssz_voluntary_exits(body.voluntary_exits)?,
        sync_aggregate: get_ssz_sync_aggregate(body.sync_aggregate)?,
        execution_payload: get_ssz_execution_payload(body.execution_payload)?,
    })
}

pub fn get_ssz_execution_payload<FeeRecipientSize: Get<u32>, LogsBloomSize: Get<u32>, ExtraDataSize: Get<u32>>(execution_payload: ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>) -> Result<SSZExecutionPayload, MerkleizationError> {
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
        extra_data: List::<u8, { config::MAX_EXTRA_DATA_BYTES }>::try_from(execution_payload.extra_data.into_inner()).map_err(|_| MerkleizationError::InvalidLength)?,
        base_fee_per_gas: U256::try_from_bytes_le(&(execution_payload.base_fee_per_gas.as_byte_slice())).map_err(|_| MerkleizationError::InvalidLength)?,
        block_hash: execution_payload.block_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        transactions_root:  execution_payload.transactions_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    };

    Ok(ssz_execution_payload)
}

pub fn get_ssz_deposits<PublicKeySize: Get<u32>, SignatureSize: Get<u32>, ProofSize: Get<u32>, DepositSize: Get<u32>>(deposits: BoundedVec<Deposit<PublicKeySize, SignatureSize, ProofSize>, DepositSize>) -> Result<List<SSZDeposit, { config::MAX_DEPOSITS }>, MerkleizationError> {
    let mut deposits_dev = Vec::new();

    for deposit in deposits.iter() {
        let mut proofs = Vec::new();

        for proof in deposit.proof.iter() {
            proofs.push(proof.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,)
        }

        let proofs_conv = Vector::<[u8; 32], {config::DEPOSIT_CONTRACT_TREE_DEPTH + 1}>::from_iter(proofs);

        deposits_dev.push(SSZDeposit{
            proof: proofs_conv,
            data: SSZDepositData{
                pubkey: Vector::<u8, 48>::from_iter(deposit.data.pubkey.clone()),
                withdrawal_credentials: deposit.data.withdrawal_credentials.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
                amount: deposit.data.amount,
                signature: Vector::<u8, 96>::from_iter(deposit.data.signature.clone()),
            }
        });
    }

    Ok(List::<SSZDeposit, { config::MAX_DEPOSITS }>::from_iter(deposits_dev))
}

pub fn get_ssz_voluntary_exits<VoluntaryExitSize: Get<u32>>(voluntary_exits: BoundedVec<VoluntaryExit, VoluntaryExitSize>) -> Result<List<SSZVoluntaryExit, { config::MAX_VOLUNTARY_EXITS }>, MerkleizationError> {
    let mut voluntary_exits_vec = Vec::new();

    for voluntary_exit in voluntary_exits.iter() {
        voluntary_exits_vec.push(SSZVoluntaryExit{
            epoch: voluntary_exit.epoch,
            validator_index: voluntary_exit.validator_index,
        });
    }

    Ok(List::<SSZVoluntaryExit, { config::MAX_VOLUNTARY_EXITS }>::from_iter(voluntary_exits_vec))
}

pub fn get_ssz_attestations<AttestionBitsSize: Get<u32>, SignatureSize: Get<u32>, AttestationSize: Get<u32>>(attestations: BoundedVec<Attestation<AttestionBitsSize, SignatureSize>, AttestationSize>) -> Result<List::<SSZAttestation, { config::MAX_ATTESTATIONS }>, MerkleizationError> {
    let mut attestations_vec = Vec::new();

    for attestation in attestations.iter() {
        attestations_vec.push(get_ssz_attestation((*attestation).clone())?);
    }

    Ok(List::<SSZAttestation, { config::MAX_ATTESTATIONS }>::from_iter(attestations_vec))
}

pub fn get_ssz_attestation<AttestionBitsSize: Get<u32>, SignatureSize: Get<u32>>(attestation: Attestation<AttestionBitsSize, SignatureSize>) -> Result<SSZAttestation, MerkleizationError> {
    let signature = Vector::<u8, 96>::from_iter(attestation.signature.clone());

    Ok(SSZAttestation{
        aggregation_bits: Bitlist::<{ config::MAX_VALIDATORS_PER_COMMITTEE }>::deserialize(&attestation.aggregation_bits).map_err(|_| MerkleizationError::InvalidLength)?,
        data: get_ssz_attestation_data(attestation.data)?,
        signature: signature,
    })
}

pub fn get_ssz_attestation_data(attestation_data: AttestationData) -> Result<SSZAttestationData, MerkleizationError> {
    Ok(SSZAttestationData{
        slot: attestation_data.slot,
        index: attestation_data.index,
        beacon_block_root: attestation_data.beacon_block_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        source: get_ssz_checkpoint(attestation_data.source)?,
        target: get_ssz_checkpoint(attestation_data.target)?,
    })
}

pub fn get_ssz_proposer_slashings<ProposerSlashingSize: Get<u32>, SignatureSize: Get<u32>>(proposer_slashings: BoundedVec<ProposerSlashing<SignatureSize>, ProposerSlashingSize>) -> Result<List<SSZProposerSlashing, { config::MAX_PROPOSER_SLASHINGS } >, MerkleizationError> {
    let mut proposer_slashings_vec = Vec::new();

    for proposer_slashing in proposer_slashings.iter() {
       proposer_slashings_vec.push(get_ssz_proposer_slashing((*proposer_slashing).clone())?);
    }

    Ok(List::<SSZProposerSlashing, { config::MAX_PROPOSER_SLASHINGS }>::from_iter(proposer_slashings_vec))
}

pub fn get_ssz_proposer_slashing<SignatureSize: Get<u32>>(proposer_slashing: ProposerSlashing<SignatureSize>) -> Result<SSZProposerSlashing, MerkleizationError> {
    let signature1 = Vector::<u8, 96>::from_iter(proposer_slashing.signed_header_1.signature.clone());
    let signature2 = Vector::<u8, 96>::from_iter(proposer_slashing.signed_header_2.signature.clone());

    Ok(SSZProposerSlashing{
        signed_header_1: SignedBeaconBlockHeader{
            message: get_ssz_beacon_header(proposer_slashing.signed_header_1.message.clone())?,
            signature: signature1,
        },
        signed_header_2: SignedBeaconBlockHeader{
            message: get_ssz_beacon_header(proposer_slashing.signed_header_2.message.clone())?,
            signature: signature2,
        },
    })
}

pub fn get_ssz_attester_slashings<AttestingIndicesSize: Get<u32>, SignatureSize: Get<u32>, AttesterSlashingSize: Get<u32>>(attester_slashings: BoundedVec<AttesterSlashing<AttestingIndicesSize, SignatureSize>, AttesterSlashingSize>) -> Result<List<SSZAttesterSlashing, { config::MAX_ATTESTER_SLASHINGS } >, MerkleizationError> {
    let mut attester_slashings_vec = Vec::new();

    for attester_slashing in attester_slashings.iter() {
        attester_slashings_vec.push(get_ssz_attester_slashing((*attester_slashing).clone())?);
    }

    Ok(List::<SSZAttesterSlashing, { config::MAX_ATTESTER_SLASHINGS } >::from_iter(attester_slashings_vec))
}

pub fn get_ssz_attester_slashing<AttestingIndicesSize: Get<u32>, SignatureSize: Get<u32>>(attester_slashing: AttesterSlashing<AttestingIndicesSize, SignatureSize>) -> Result<SSZAttesterSlashing, MerkleizationError> {
    let signature1 = Vector::<u8, 96>::from_iter(attester_slashing.attestation_1.signature.clone());
    let signature2 = Vector::<u8, 96>::from_iter(attester_slashing.attestation_2.signature.clone());

    let attesting_indices1 = List::<u64, { config::MAX_VALIDATORS_PER_COMMITTEE }>::from_iter(attester_slashing.attestation_1.attesting_indices.clone());
    let attesting_indices2 = List::<u64, { config::MAX_VALIDATORS_PER_COMMITTEE }>::from_iter(attester_slashing.attestation_2.attesting_indices.clone());

    Ok(SSZAttesterSlashing{
        attestation_1: SSZIndexedAttestation{
            attesting_indices: attesting_indices1,
            data: get_ssz_attestation_data(attester_slashing.attestation_1.data.clone())?,
            signature: signature1,
        },
        attestation_2: SSZIndexedAttestation{
            attesting_indices: attesting_indices2,
            data: get_ssz_attestation_data(attester_slashing.attestation_2.data.clone())?,
            signature: signature2,
        },
    })
}

pub fn get_ssz_checkpoint(checkpoint: Checkpoint) -> Result<SSZCheckpoint, MerkleizationError> {
    Ok(SSZCheckpoint{
        epoch: checkpoint.epoch,
        root: checkpoint.root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    })
}

pub fn get_ssz_beacon_header(beacon_header: BeaconHeader) -> Result<SSZBeaconBlockHeader, MerkleizationError> {
    Ok(SSZBeaconBlockHeader{
        slot: beacon_header.slot,
        proposer_index: beacon_header.proposer_index,
        parent_root: beacon_header.parent_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        state_root: beacon_header.state_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        body_root: beacon_header.body_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    })
}

pub fn get_ssz_sync_aggregate<SyncCommitteeBitsSize: Get<u32>, SignatureSize: Get<u32>>(sync_aggregate: SyncAggregate<SyncCommitteeBitsSize, SignatureSize>) -> Result<SSZSyncAggregate, MerkleizationError> {

    Ok(SSZSyncAggregate{
        sync_committee_bits: Bitvector::<{ config::SYNC_COMMITTEE_SIZE }>::deserialize(&sync_aggregate.sync_committee_bits).map_err(|_| MerkleizationError::InvalidLength)?,
        sync_committee_signature: Vector::<u8, 96>::from_iter(sync_aggregate.sync_committee_signature),
    })
}

pub fn get_ssz_eth1_data(eth1_data: Eth1Data) -> Result<SSZEth1Data, MerkleizationError> {
    Ok(SSZEth1Data{
        deposit_root: eth1_data.deposit_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        deposit_count: eth1_data.deposit_count,
        block_hash: eth1_data.block_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    })
}

pub fn hash_tree_root_beacon_header(beacon_header: BeaconHeader) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(get_ssz_beacon_header(beacon_header)?)
}

pub fn hash_tree_root_beacon_body<
	FeeRecipientSize: Get<u32>, 
	LogsBloomSize: Get<u32>, 
	ExtraDataSize: Get<u32>, 
	DepositDataSize: Get<u32>, 
	PublicKeySize: Get<u32>, 
	SignatureSize: Get<u32>, 
	ProofSize: Get<u32>, 
	ProposerSlashingSize: Get<u32>, 
	AttesterSlashingSize: Get<u32>, 
	VoluntaryExitSize: Get<u32>,
	AttestationSize: Get<u32>,
	ValidatorCommitteeSize: Get<u32>,
    SyncCommitteeSize: Get<u32>>(body: Body<FeeRecipientSize, 
	LogsBloomSize, 
	ExtraDataSize, 
	DepositDataSize, 
	PublicKeySize, 
	SignatureSize, 
	ProofSize, 
	ProposerSlashingSize, 
	AttesterSlashingSize, 
	VoluntaryExitSize,
	AttestationSize,
	ValidatorCommitteeSize,
    SyncCommitteeSize>) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(get_ssz_beacon_block_body(body)?)
}

pub fn hash_tree_root_sync_committee<S: Get<u32>>(sync_committee: SyncCommittee<S>) -> Result<[u8; 32], MerkleizationError> {
    let mut pubkeys_vec = Vec::new();

    for pubkey in sync_committee.pubkeys.iter() {
        let conv_pubkey = Vector::<u8, 48>::from_iter(pubkey.0);

        pubkeys_vec.push(conv_pubkey);
    }

    let pubkeys = Vector::<Vector::<u8, 48>, { config::SYNC_COMMITTEE_SIZE }>::from_iter(pubkeys_vec.clone());

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

pub fn get_sync_committee_bits<SyncCommitteeBitsSize: Get<u32>>(bits_hex: BoundedVec<u8, SyncCommitteeBitsSize>) -> Result<Vec<u8>, MerkleizationError> {
    let bitv = Bitvector::<{ config::SYNC_COMMITTEE_SIZE }>::deserialize(&bits_hex).map_err(|e| -> MerkleizationError {
        match e {
            DeserializeError::InputTooShort => MerkleizationError::InputTooShort,
            DeserializeError::ExtraInput => MerkleizationError::ExtraInput,
            _ => MerkleizationError::InvalidInput,
        }
    })?;

    let result = bitv.iter().map(|bit| {
        if bit == true {
            1
        } else {
            0
        }
    }).collect::<Vec<_>>();

    Ok(result)
}
