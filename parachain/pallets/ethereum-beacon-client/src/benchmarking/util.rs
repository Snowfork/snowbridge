use crate::{
	decompress_sync_committee_bits, Config, CurrentSyncCommittee, Pallet as EthereumBeaconClient,
	Update, ValidatorsRoot, Vec,
};
use primitives::PublicKeyPrepared;
use sp_core::H256;

use super::fixtures::{make_checkpoint, make_sync_committee_update};

pub fn initialize_sync_committee<T: Config>() -> Result<Update, &'static str> {
	let initial_sync_data = make_checkpoint();

	EthereumBeaconClient::<T>::process_checkpoint_update(&initial_sync_data)?;

	let sync_committee_update = make_sync_committee_update();

	Ok(sync_committee_update)
}

pub fn participant_pubkeys<T: Config>(
	update: &Update,
) -> Result<Vec<PublicKeyPrepared>, &'static str> {
	let sync_committee_bits =
		decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
	let current_sync_committee = <CurrentSyncCommittee<T>>::get();
	let pubkeys = EthereumBeaconClient::<T>::find_pubkeys(
		&sync_committee_bits,
		&current_sync_committee.pubkeys.as_ref(),
		true,
	);
	Ok(pubkeys)
}

pub fn absent_pubkeys<T: Config>(update: &Update) -> Result<Vec<PublicKeyPrepared>, &'static str> {
	let sync_committee_bits =
		decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
	let current_sync_committee = <CurrentSyncCommittee<T>>::get();
	let pubkeys = EthereumBeaconClient::<T>::find_pubkeys(
		&sync_committee_bits,
		&current_sync_committee.pubkeys.as_ref(),
		false,
	);
	Ok(pubkeys)
}

pub fn signing_root<T: Config>(update: &Update) -> Result<H256, &'static str> {
	let validators_root = <ValidatorsRoot<T>>::get();
	let signing_root = EthereumBeaconClient::<T>::signing_root(
		&update.attested_header,
		validators_root,
		update.signature_slot,
	)?;
	Ok(signing_root)
}
