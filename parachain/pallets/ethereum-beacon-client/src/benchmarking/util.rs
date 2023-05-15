use crate::{
	decompress_sync_committee_bits, Config, LatestSyncCommitteePeriod,
	Pallet as EthereumBeaconClient, SyncCommitteeUpdate, SyncCommittees, ValidatorsRoot, Vec,
	SYNC_COMMITTEE_SIZE,
};
use primitives::{PublicKeyPrepared, SyncCommitteePrepared};
use sp_core::H256;

use super::{initial_sync, sync_committee_update};

pub fn initialize_sync_committee<T: Config>() -> Result<SyncCommitteeUpdate, &'static str> {
	let initial_sync_data = initial_sync();

	EthereumBeaconClient::<T>::process_checkpoint_update(&initial_sync_data)?;

	let sync_committee_update = sync_committee_update();

	//initialize SyncCommittees with period in sync_committee_update
	LatestSyncCommitteePeriod::<T>::set(EthereumBeaconClient::<T>::compute_current_sync_period(
		sync_committee_update.attested_header.slot,
	));
	let current_period = EthereumBeaconClient::<T>::compute_current_sync_period(
		sync_committee_update.attested_header.slot,
	);
	EthereumBeaconClient::<T>::store_sync_committee(
		current_period,
		&initial_sync_data.current_sync_committee,
	)?;
	Ok(sync_committee_update)
}

pub fn sync_committee<T: Config>(
	update: &SyncCommitteeUpdate,
) -> Result<SyncCommitteePrepared<SYNC_COMMITTEE_SIZE>, &'static str> {
	let current_period =
		EthereumBeaconClient::<T>::compute_current_sync_period(update.attested_header.slot);
	let sync_committee = SyncCommittees::<T>::get(current_period).ok_or("no sync committee")?;
	Ok(sync_committee)
}

pub fn participant_pubkeys<T: Config>(
	update: &SyncCommitteeUpdate,
) -> Result<Vec<PublicKeyPrepared>, &'static str> {
	let sync_committee_bits =
		decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
	let current_sync_committee = sync_committee::<T>(update)?;
	let pubkeys = EthereumBeaconClient::<T>::find_pubkeys(
		&sync_committee_bits,
		&current_sync_committee.pubkeys.as_ref(),
		true,
	);
	Ok(pubkeys)
}

pub fn absent_pubkeys<T: Config>(
	update: &SyncCommitteeUpdate,
) -> Result<Vec<PublicKeyPrepared>, &'static str> {
	let sync_committee_bits =
		decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
	let current_sync_committee = sync_committee::<T>(update)?;
	let pubkeys = EthereumBeaconClient::<T>::find_pubkeys(
		&sync_committee_bits,
		&current_sync_committee.pubkeys.as_ref(),
		false,
	);
	Ok(pubkeys)
}

pub fn signing_root<T: Config>(update: &SyncCommitteeUpdate) -> Result<H256, &'static str> {
	let validators_root = <ValidatorsRoot<T>>::get();
	let signing_root = EthereumBeaconClient::<T>::signing_root(
		update.attested_header,
		validators_root,
		update.signature_slot,
	)?;
	Ok(signing_root)
}
