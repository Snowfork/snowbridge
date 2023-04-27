use super::*;
use crate::Pallet as EthereumBeaconClient;
use milagro_bls::{AggregatePublicKey, AggregateSignature, Signature};

pub fn initialize_sync_committee<T: Config>() -> Result<SyncCommitteeUpdateOf<T>, &'static str> {
	let initial_sync_data = initial_sync();

	EthereumBeaconClient::<T>::initial_sync(initial_sync_data.clone())?;

	let sync_committee_update: SyncCommitteeUpdateOf<T> = sync_committee_update();

	//initialize SyncCommittees with period in sync_committee_update
	LatestSyncCommitteePeriod::<T>::set(EthereumBeaconClient::<T>::compute_current_sync_period(
		sync_committee_update.attested_header.slot,
	));
	SyncCommittees::<T>::insert(
		EthereumBeaconClient::<T>::compute_current_sync_period(
			sync_committee_update.attested_header.slot,
		),
		initial_sync_data.current_sync_committee,
	);
	Ok(sync_committee_update)
}

pub fn get_participant_pubkeys<T: Config>(
	update: &SyncCommitteeUpdateOf<T>,
) -> Result<Vec<PublicKey>, &'static str> {
	let sync_committee_bits =
		get_sync_committee_bits(update.sync_aggregate.sync_committee_bits.clone()).unwrap();
	let current_period =
		EthereumBeaconClient::<T>::compute_current_sync_period(update.attested_header.slot);
	let current_sync_committee =
		EthereumBeaconClient::<T>::get_sync_committee_for_period(current_period)?;
	let sync_committee_pubkeys = current_sync_committee.pubkeys;
	let mut participant_pubkeys: Vec<PublicKey> = Vec::new();
	for (bit, pubkey) in sync_committee_bits.iter().zip(sync_committee_pubkeys.iter()) {
		if *bit == 1 as u8 {
			let pubk = pubkey.clone();
			participant_pubkeys.push(pubk);
		}
	}
	Ok(participant_pubkeys)
}

pub fn get_signing_message<T: Config>(
	update: &SyncCommitteeUpdateOf<T>,
) -> Result<Root, &'static str> {
	let validators_root = <ValidatorsRoot<T>>::get();
	let fork_version = EthereumBeaconClient::<T>::compute_fork_version(
		EthereumBeaconClient::<T>::compute_epoch_at_slot(
			update.signature_slot,
			config::SLOTS_PER_EPOCH,
		),
	);
	let domain_type = config::DOMAIN_SYNC_COMMITTEE.to_vec();
	let domain =
		EthereumBeaconClient::<T>::compute_domain(domain_type, fork_version, validators_root)?;
	let signing_root =
		EthereumBeaconClient::<T>::compute_signing_root(update.attested_header.clone(), domain)?;
	Ok(signing_root)
}

pub fn get_aggregate_signature<T: Config>(
	signature: BoundedVec<u8, T::MaxSignatureSize>,
) -> Result<AggregateSignature, Error<T>> {
	let sig = Signature::from_bytes(&signature[..]).map_err(|_| Error::<T>::InvalidSignature)?;
	let agg_sig = AggregateSignature::from_signature(&sig);
	Ok(agg_sig)
}

pub fn get_aggregate_pubkey<T: Config>(
	pubkeys: Vec<PublicKey>,
) -> Result<AggregatePublicKey, Error<T>> {
	let milagro_public_keys = pubkeys
		.iter()
		.map(|bytes| milagro_bls::PublicKey::from_bytes_unchecked(&bytes.0))
		.collect::<Result<Vec<milagro_bls::PublicKey>, _>>()
		.map_err(|_| Error::<T>::InvalidSignaturePoint)?;
	let agg_pub_key = AggregatePublicKey::into_aggregate(&milagro_public_keys)
		.map_err(|_| Error::<T>::InvalidAggregatePublicKeys)?;
	Ok(agg_pub_key)
}
