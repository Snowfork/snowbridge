// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{
	decompress_sync_committee_bits, types::ArkSyncCommitteePrepared, Config, CurrentSyncCommittee,
	Pallet as EthereumBeaconClient, Update, ValidatorsRoot, Vec,
};
use codec::Decode;
use primitives::ArkPublicKeyPrepared;
use sp_core::H256;

pub fn participant_pubkeys<T: Config>(
	update: &Update,
) -> Result<Vec<ArkPublicKeyPrepared>, &'static str> {
	let sync_committee_bits =
		decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
	let current_sync_committee_raw = <CurrentSyncCommittee<T>>::get();
	let current_sync_committee =
		ArkSyncCommitteePrepared::decode(&mut &current_sync_committee_raw[..]).unwrap();
	let pubkeys = EthereumBeaconClient::<T>::find_pubkeys(
		&sync_committee_bits,
		(*current_sync_committee.pubkeys).as_ref(),
		true,
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
