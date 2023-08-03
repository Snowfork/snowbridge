use std::convert::TryInto;
use snowbridge_beacon_primitives::types::BeaconHeader;
use snowbridge_beacon_primitives::updates::AncestryProof;
use snowbridge_ethereum_beacon_client::types::{SyncCommittee, CheckpointUpdate, Update, NextSyncCommitteeUpdate, SyncAggregate};
use snowbridge_beacon_primitives::{PublicKey, ExecutionPayloadHeader, ExecutionHeaderUpdate};
use sp_core::H256;
use crate::types::{FuzzExecutionHeaderUpdate, FuzzSyncAggregate, FuzzNextSyncCommitteeUpdate,
                   FuzzUpdate, FuzzSyncCommittee, FuzzAncestryProof, FuzzExecutionPayloadHeader, FuzzBeaconHeader, FuzzCheckpointUpdate};

impl TryFrom<FuzzUpdate> for Update
{
    type Error = String;

    fn try_from(other: FuzzUpdate) -> Result<Self, Self::Error> {
        let next: Option<NextSyncCommitteeUpdate> = other.next_sync_committee_update.map(|fuzz_update| fuzz_update.try_into().unwrap());

        Ok(Self {
            attested_header: other.attested_header.clone().try_into().unwrap(),
            sync_aggregate: other.sync_aggregate.try_into().unwrap(),
            signature_slot: other.signature_slot,
            next_sync_committee_update: next,
            finalized_header: other.finalized_header.clone().try_into().unwrap(),
            finality_branch: other.finality_branch.iter().map(|&hash| {
                H256::from(hash)
            }).collect::<Vec<_>>().as_slice().try_into().unwrap(),
            block_roots_root: other.block_roots_root.into(),
            block_roots_branch: other.block_roots_branch.iter().map(|&hash| {
                H256::from(hash)
            }).collect::<Vec<_>>().as_slice().try_into().unwrap(),
        })
    }
}

impl TryFrom<FuzzNextSyncCommitteeUpdate> for NextSyncCommitteeUpdate
{
    type Error = String;

    fn try_from(other: FuzzNextSyncCommitteeUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            next_sync_committee: SyncCommittee{
                pubkeys: other.next_sync_committee.pubkeys.iter().map(|&pk| {
                    let p: PublicKey = pk.into();
                    p
                }).collect::<Vec<_>>().as_slice().try_into().unwrap(),
                aggregate_pubkey: other.next_sync_committee.aggregate_pubkey.into(),
            },
            next_sync_committee_branch: other.next_sync_committee_branch.iter().map(|&hash| {
                H256::from(hash)
            }).collect::<Vec<_>>().as_slice().try_into().unwrap(),
        })
    }
}

impl TryFrom<FuzzSyncAggregate> for SyncAggregate
{
    type Error = String;

    fn try_from(other: FuzzSyncAggregate) -> Result<Self, Self::Error> {
        Ok(Self {
            sync_committee_bits: other.sync_committee_bits.into(),
            sync_committee_signature: other.sync_committee_signature.into(),
        })
    }
}

impl TryFrom<FuzzCheckpointUpdate> for CheckpointUpdate
{
    type Error = String;

    fn try_from(other: FuzzCheckpointUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            header: other.header.clone().try_into().unwrap(),
            current_sync_committee: other.current_sync_committee.try_into().unwrap(),
            current_sync_committee_branch: other.current_sync_committee_branch.iter().map(|&hash| {
                H256::from(hash)
            }).collect::<Vec<_>>().as_slice().try_into().unwrap(),
            validators_root: other.validators_root.into(),
            block_roots_root: other.block_roots_root.into(),
            block_roots_branch: other.block_roots_branch.iter().map(|&hash| {
                H256::from(hash)
            }).collect::<Vec<_>>().as_slice().try_into().unwrap(),
        })
    }
}

impl TryFrom<FuzzSyncCommittee> for SyncCommittee
{
    type Error = String;

    fn try_from(other: FuzzSyncCommittee) -> Result<Self, Self::Error> {
        Ok(Self{
            pubkeys: other.pubkeys.iter().map(|&pk| {
                let p: PublicKey = pk.into();
                p
            }).collect::<Vec<_>>().as_slice().try_into().unwrap(),
            aggregate_pubkey: other.aggregate_pubkey.into(),
        })
    }
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
