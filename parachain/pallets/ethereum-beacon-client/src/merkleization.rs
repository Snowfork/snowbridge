use ssz_rs::{Deserialize, SimpleSerialize as SimpleSerializeTrait, Bitlist, Bitvector};
use ssz_rs::prelude::{Vector, List};
use sp_std::convert::TryInto;
use sp_std::iter::FromIterator;
use sp_std::prelude::*;
use ssz_rs::U256;
use byte_slice_cast::AsByteSlice;
use snowbridge_beacon::{SyncAggregate, Attestation, Checkpoint, Eth1Data, BeaconHeader, AttesterSlashing, ExecutionPayload, SigningData, ForkData, SyncCommittee, AttestationData, Body, ProposerSlashing, Deposit, VoluntaryExit};
use crate::ssz::*;
use crate::config;

#[derive(Debug)]
pub enum MerkleizationError {
    HashTreeRootError,
    HashTreeRootInvalidBytes,
    InvalidLength
}

pub fn get_ssz_beacon_block_body(body: Body) -> Result<SSZBeaconBlockBody, MerkleizationError> {
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

pub fn get_ssz_execution_payload(execution_payload: ExecutionPayload) -> Result<SSZExecutionPayload, MerkleizationError> {
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
        extra_data: List::<u8, { config::MAX_EXTRA_DATA_BYTES }>::try_from(execution_payload.extra_data).map_err(|_| MerkleizationError::InvalidLength)?,
        base_fee_per_gas: U256::try_from_bytes_le(&(execution_payload.base_fee_per_gas.as_byte_slice())).map_err(|_| MerkleizationError::InvalidLength)?,
        block_hash: execution_payload.block_hash.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
        transactions_root:  execution_payload.transactions_root.as_bytes().try_into().map_err(|_| MerkleizationError::InvalidLength)?,
    };

    Ok(ssz_execution_payload)
}

pub fn get_ssz_deposits(deposits: Vec<Deposit>) -> Result<List<SSZDeposit, { config::MAX_DEPOSITS }>, MerkleizationError> {
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

pub fn get_ssz_voluntary_exits(voluntary_exits: Vec<VoluntaryExit>) -> Result<List<SSZVoluntaryExit, { config::MAX_VOLUNTARY_EXITS }>, MerkleizationError> {
    let mut voluntary_exits_vec = Vec::new();

    for voluntary_exit in voluntary_exits.iter() {
        voluntary_exits_vec.push(SSZVoluntaryExit{
            epoch: voluntary_exit.epoch,
            validator_index: voluntary_exit.validator_index,
        });
    }

    Ok(List::<SSZVoluntaryExit, { config::MAX_VOLUNTARY_EXITS }>::from_iter(voluntary_exits_vec))
}

pub fn get_ssz_attestations(attestations: Vec<Attestation>) -> Result<List::<SSZAttestation, { config::MAX_ATTESTATIONS }>, MerkleizationError> {
    let mut attestations_vec = Vec::new();

    for attestation in attestations.iter() {
        attestations_vec.push(get_ssz_attestation((*attestation).clone())?);
    }

    Ok(List::<SSZAttestation, { config::MAX_ATTESTATIONS }>::from_iter(attestations_vec))
}

pub fn get_ssz_attestation(attestation: Attestation) -> Result<SSZAttestation, MerkleizationError> {
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

pub fn get_ssz_proposer_slashings(proposer_slashings: Vec<ProposerSlashing>) -> Result<List<SSZProposerSlashing, { config::MAX_PROPOSER_SLASHINGS } >, MerkleizationError> {
    let mut proposer_slashings_vec = Vec::new();

    for proposer_slashing in proposer_slashings.iter() {
       proposer_slashings_vec.push(get_ssz_proposer_slashing((*proposer_slashing).clone())?);
    }

    Ok(List::<SSZProposerSlashing, { config::MAX_PROPOSER_SLASHINGS }>::from_iter(proposer_slashings_vec))
}

pub fn get_ssz_proposer_slashing(proposer_slashing: ProposerSlashing) -> Result<SSZProposerSlashing, MerkleizationError> {
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

pub fn get_ssz_attester_slashings(attester_slashings: Vec<AttesterSlashing>) -> Result<List<SSZAttesterSlashing, { config::MAX_ATTESTER_SLASHINGS } >, MerkleizationError> {
    let mut attester_slashings_vec = Vec::new();

    for attester_slashing in attester_slashings.iter() {
        attester_slashings_vec.push(get_ssz_attester_slashing((*attester_slashing).clone())?);
    }

    Ok(List::<SSZAttesterSlashing, { config::MAX_ATTESTER_SLASHINGS } >::from_iter(attester_slashings_vec))
}

pub fn get_ssz_attester_slashing(attester_slashing: AttesterSlashing) -> Result<SSZAttesterSlashing, MerkleizationError> {
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

pub fn get_ssz_sync_aggregate(sync_aggregate: SyncAggregate) -> Result<SSZSyncAggregate, MerkleizationError> {
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

pub fn hash_tree_root_beacon_body(body: Body) -> Result<[u8; 32], MerkleizationError> {
    hash_tree_root(get_ssz_beacon_block_body(body)?)
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

pub fn get_sync_committee_bits(bits_hex: Vec<u8>) -> Result<Vec<u8>, MerkleizationError> {
    let bitv = Bitvector::<{ config::SYNC_COMMITTEE_SIZE }>::deserialize(&bits_hex).map_err(|_| MerkleizationError::InvalidLength)?;

    let mut result = Vec::new();

    for bit in bitv.iter() {
        if bit == true {
            result.push(1);
        } else {
            result.push(0);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use snowbridge_beacon::{AttestationData, Checkpoint, Eth1Data, Attestation, ExecutionPayload, SyncAggregate};
    use crate::merkleization;
    use crate as ethereum_beacon_client;
    use frame_support::{assert_ok};
    use hex_literal::hex;
    use ssz_rs::prelude::Vector;
    use sp_core::U256;
    use crate::mock::{get_attester_slashing, get_block_body, get_sync_committee};

    #[test]
    pub fn test_hash_tree_root_beacon_header() {
        let hash_root = merkleization::hash_tree_root_beacon_header(
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
    pub fn test_hash_tree_root_beacon_header_more_test_values() {
        let hash_root = merkleization::hash_tree_root_beacon_header(
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
        let hash_root = merkleization::hash_tree_root_sync_committee(
            get_sync_committee()
        );

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("24409c991386e5d43bcecf871dc1fa395013f0293c86766877f745a408148a3a")
        );
    }

    #[test]
    pub fn test_hash_tree_root_fork_data() {
        let hash_root = merkleization::hash_tree_root_fork_data(
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
        let hash_root = merkleization::hash_tree_root_signing_data(
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
    pub fn test_hash_block_body() {
        let payload = merkleization::get_ssz_beacon_block_body(
            get_block_body()
        );

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_eq!(
            hash_root.unwrap(),
            hex!("f7c410dbb50104bd95a66ad2b564cc18bbcce06391e572b61968a2d78c3ee4cc")
        );
    }

    #[test]
    pub fn test_hash_eth1_data() {
        let payload = merkleization::get_ssz_eth1_data(Eth1Data{
            deposit_root: hex!("d70a234731285c6804c2a4f56711ddb8c82c99740f207854891028af34e27e5e").into(),
            deposit_count: 0,
            block_hash: hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
        });

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_eq!(
            hash_root.unwrap(),
            hex!("aa247f2dfbb6e5d77b7e9f637f9bb70842cbec34cb4238d5bcb491f4e4b3fa5e")
        );
    }

    #[test]
    pub fn test_hash_sync_aggregrate() {
        let payload = merkleization::get_ssz_sync_aggregate(SyncAggregate{
            sync_committee_bits: hex!("cefffffefffffff767fffbedffffeffffeeffdffffdebffffff7f7dbdf7fffdffffbffcfffdff79dfffbbfefff2ffffff7ddeff7ffffc98ff7fbfffffffffff7").to_vec(),
            sync_committee_signature: hex!("8af1a8577bba419fe054ee49b16ed28e081dda6d3ba41651634685e890992a0b675e20f8d9f2ec137fe9eb50e838aa6117f9f5410e2e1024c4b4f0e098e55144843ce90b7acde52fe7b94f2a1037342c951dc59f501c92acf7ed944cb6d2b5f7").to_vec(),
        });

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_eq!(
            hash_root.unwrap(),
            hex!("e6dcad4f60ce9ff8a587b110facbaf94721f06cd810b6d8bf6cffa641272808d")
        );
    }

    #[test]
    pub fn test_hash_sync_signature() {
        let payload = Vector::<u8, 96>::from_iter(hex!("82c58d251044ab938b84747524e9b5ecbf6f71f6f1ac10a834806d033bbc49ecd2391072f9bbb4758a960342f8ee03930dc8195f15649c654a56767632230fe3d196f6499d94cd239ba964fe21d7e4715127a385ee018d405719428178172188").to_vec());

        let hash_root = merkleization::hash_tree_root(payload);

        assert_eq!(
            hash_root.unwrap(),
            hex!("2068ede33715fd1eee4a940cea6ebc7d353ea791c18ed0cdc65ab6f4bd367af1")
        );
    }

    #[test]
    pub fn test_hash_tree_root_execution_payload() {
        let payload = merkleization::get_ssz_execution_payload(
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
                base_fee_per_gas: U256::from(7 as i16),
                block_hash: hex!("cd8df91b4503adb8f2f1c7a4f60e07a1f1a2cbdfa2a95bceba581f3ff65c1968").into(),
                transactions_root: hex!("7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1").into(),
            }
        );

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_eq!(
            hash_root.unwrap(),
            hex!("4c74e6119faeee22c04ef02fb6d8db26799753e2a9efcde6ea60cbac1f38cfd2")
        );
    }

    #[test]
    pub fn test_hash_tree_root_attestation() {
        let payload = merkleization::get_ssz_attestation(
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

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("a60acb46465c9eda6047e2cc18b3d509b7610efcbc7a02d28aea3ffa67e89f5a")
        );
    }

    #[test]
    pub fn test_hash_tree_root_attestation_data() {
        let payload = merkleization::get_ssz_attestation_data(
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

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("351d24efe677a40e3b687f8c95821158c3a3bb7c41c43b51187d4c1df690c849")
        );
    }

    #[test]
    pub fn test_hash_tree_root_checkpoint() {
        let payload = merkleization::get_ssz_checkpoint(
            Checkpoint{
                epoch: 15127,
                root: hex!("e665df84b5f1b4db9112b5c3876f5c10063347bfaf1025732137cf9abca28b75").into(),
            }
        );

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("c83bfcaa363a349b6869d70dcfe430f6199f8da7b01eb92d05a0860efe19dcec")
        );
    }

    #[test]
    pub fn test_hash_tree_root_attester_slashing() {
        let payload = merkleization::get_ssz_attester_slashing(
           get_attester_slashing()
        );

        assert_ok!(&payload);

        let hash_root = merkleization::hash_tree_root(payload.unwrap());

        assert_ok!(&hash_root);
        assert_eq!(
            hash_root.unwrap(),
            hex!("4c647fb5557d5a443eda8eeded902901cf0e0d3bff9be7f8764d613918fcfe0d")
        );
    }
}
