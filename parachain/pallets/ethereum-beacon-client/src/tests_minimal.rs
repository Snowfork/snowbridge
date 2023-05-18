use crate::{
	config,
	config::{SYNC_COMMITTEE_BITS_SIZE, SYNC_COMMITTEE_SIZE},
	mock::*,
	pallet::FinalizedBeaconHeadersBlockRoot,
	Error, ExecutionHeaderState, ExecutionHeaders, FinalizedBeaconHeaders, FinalizedHeaderState,
	LatestExecutionHeader, LatestFinalizedHeader, ValidatorsRoot,
};
use frame_support::{assert_err, assert_ok};
use hex_literal::hex;
use primitives::{
	decompress_sync_committee_bits, fast_aggregate_verify_legacy, prepare_g1_pubkeys,
};
use sp_core::H256;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn it_syncs_from_an_initial_checkpoint() {
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(&initial_sync));

		let block_root: H256 = initial_sync.header.hash_tree_root().unwrap();

		assert!(<FinalizedBeaconHeaders<mock_minimal::Test>>::contains_key(block_root));
	});
}

#[test]
fn it_updates_a_committee_period_sync_update() {
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

	let update =
		get_committee_sync_period_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(&initial_sync));

		let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
			update.attested_header.slot,
		);

		assert_ok!(mock_minimal::EthereumBeaconClient::store_sync_committee(
			current_period,
			&initial_sync.current_sync_committee,
		));

		assert_ok!(mock_minimal::EthereumBeaconClient::sync_committee_period_update(
			mock_minimal::RuntimeOrigin::signed(1),
			update.clone(),
		));

		let block_root: H256 = update.finalized_header.hash_tree_root().unwrap();

		assert!(<FinalizedBeaconHeaders<mock_minimal::Test>>::contains_key(block_root));
	});
}

#[test]
fn it_updates_a_committee_period_sync_update_with_invalid_signature_slot() {
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

	let mut update =
		get_committee_sync_period_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(&initial_sync));

		// makes a invalid update with signature_slot should be more than attested_slot
		update.signature_slot = update.attested_header.slot;

		assert_err!(
			mock_minimal::EthereumBeaconClient::sync_committee_period_update(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone(),
			),
			Error::<mock_minimal::Test>::InvalidSignatureSlot
		);
	});
}

/*
#[test]
fn it_updates_a_invalid_committee_period_sync_update_with_gap() {
	let initial_sync = get_initial_sync::<mock_minimal::Test>();

	let update = get_committee_sync_period_update::<mock_minimal::Test>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::initial_sync(initial_sync.clone()));

		let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
			update.attested_header.slot,
		);

		SyncCommittees::<mock_minimal::Test>::insert(
			current_period,
			initial_sync.current_sync_committee,
		);

		assert_err!(
			mock_minimal::EthereumBeaconClient::sync_committee_period_update(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone(),
			),
			Error::<mock_minimal::Test>::InvalidSyncCommitteeUpdateWithGap
		);
	});
}
*/

#[test]
fn it_updates_a_invalid_committee_period_sync_update_with_duplicate_entry() {
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

	let update =
		get_committee_sync_period_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(&initial_sync));

		let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
			update.attested_header.slot,
		);

		assert_ok!(mock_minimal::EthereumBeaconClient::store_sync_committee(
			current_period,
			&initial_sync.current_sync_committee,
		));

		// initialize with period of the next update
		assert_ok!(mock_minimal::EthereumBeaconClient::store_sync_committee(
			current_period + 1,
			&initial_sync.current_sync_committee,
		));

		assert_err!(
			mock_minimal::EthereumBeaconClient::sync_committee_period_update(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone(),
			),
			Error::<mock_minimal::Test>::InvalidSyncCommitteeUpdateWithDuplication
		);
	});
}

#[test]
fn it_processes_a_finalized_header_update() {
	let update = get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

	let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
		update.attested_header.slot,
	);

	let time_now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("Time went backwards")
		.as_secs();

	let import_time = time_now + (update.finalized_header.slot * config::SECONDS_PER_SLOT as u64); // Goerli genesis time + finalized header update time
	let mock_pallet_time = import_time + 3600; // plus one hour

	new_tester::<mock_minimal::Test>().execute_with(|| {
		mock_minimal::Timestamp::set_timestamp(mock_pallet_time * 1000); // needs to be in milliseconds
		LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
			beacon_block_root: Default::default(),
			import_time,
			// set the last imported finalized header to an older finalized header. Necessary
			// for long range attack check and finalized header to be imported must not have
			// been imported already.
			beacon_slot: update.finalized_header.slot - 1,
		});
		ValidatorsRoot::<mock_minimal::Test>::set(get_validators_root::<SYNC_COMMITTEE_SIZE>());
		assert_ok!(mock_minimal::EthereumBeaconClient::store_sync_committee(
			current_period,
			&initial_sync.current_sync_committee,
		));

		assert_ok!(mock_minimal::EthereumBeaconClient::import_finalized_header(
			mock_minimal::RuntimeOrigin::signed(1),
			update.clone()
		));

		let block_root: H256 = update.finalized_header.clone().hash_tree_root().unwrap();

		assert!(<FinalizedBeaconHeaders<mock_minimal::Test>>::contains_key(block_root));
	});
}

#[test]
fn it_processes_a_invalid_finalized_header_update() {
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();
	let update = get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(&initial_sync));

		LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
			beacon_block_root: Default::default(),
			import_time: 0,
			// initialize with the same slot as the next updating
			beacon_slot: update.finalized_header.slot,
		});

		// update with same slot as last finalized will fail
		assert_err!(
			mock_minimal::EthereumBeaconClient::import_finalized_header(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone()
			),
			Error::<mock_minimal::Test>::SyncCommitteeMissing
		);
	});
}

/*
#[test]
fn it_processes_a_invalid_finalized_header_update_with_period_gap() {
	let initial_sync = get_initial_sync::<mock_minimal::Test>();
	let update = get_finalized_header_update::<mock_minimal::Test>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		LatestFinalizedHeaderState::<mock_minimal::Test>::set(FinalizedHeaderState {
			beacon_block_root: Default::default(),
			import_time: 0,
			// initialize last_finalized_slot as period 0
			beacon_slot: 1,
		});
		SyncCommittees::<mock_minimal::Test>::insert(0, initial_sync.current_sync_committee);
		ValidatorsRoot::<mock_minimal::Test>::set(get_validators_root::<mock_minimal::Test>());

		// update with period 2 to make period gap check fail
		assert_err!(
			mock_minimal::EthereumBeaconClient::import_finalized_header(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone()
			),
			Error::<mock_minimal::Test>::InvalidFinalizedPeriodUpdate
		);
	});
}*/

#[test]
fn it_processes_a_header_update() {
	let update = get_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

	let current_sync_committee = get_initial_sync::<SYNC_COMMITTEE_SIZE>().current_sync_committee;

	let finalized_update =
		get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
	let finalized_slot = finalized_update.finalized_header.slot;
	let finalized_block_root: H256 = finalized_update.finalized_header.hash_tree_root().unwrap();

	let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
		update.attested_header.slot,
	);

	new_tester::<mock_minimal::Test>().execute_with(|| {
		ValidatorsRoot::<mock_minimal::Test>::set(get_validators_root::<SYNC_COMMITTEE_SIZE>());
		LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
			beacon_block_root: finalized_block_root,
			beacon_slot: finalized_slot,
			import_time: 0,
		});
		FinalizedBeaconHeadersBlockRoot::<mock_minimal::Test>::insert(
			finalized_block_root,
			finalized_update.block_roots_root,
		);
		assert_ok!(mock_minimal::EthereumBeaconClient::store_sync_committee(
			current_period,
			&current_sync_committee,
		));

		assert_ok!(mock_minimal::EthereumBeaconClient::import_execution_header(
			mock_minimal::RuntimeOrigin::signed(1),
			update.clone()
		));

		assert!(<ExecutionHeaders<mock_minimal::Test>>::contains_key(
			update.execution_header.block_hash
		));
	});
}

#[test]
fn it_processes_a_invalid_header_update_not_finalized() {
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();
	let update = get_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
	let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
		update.attested_header.slot,
	);

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(&initial_sync));

		LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
			beacon_block_root: H256::default(),
			// initialize finalized state with parent slot of the next update
			beacon_slot: update.attested_header.slot - 1,
			import_time: 0,
		});

		assert_ok!(mock_minimal::EthereumBeaconClient::store_sync_committee(
			current_period,
			&initial_sync.current_sync_committee,
		));

		assert_err!(
			mock_minimal::EthereumBeaconClient::import_execution_header(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone()
			),
			Error::<mock_minimal::Test>::HeaderNotFinalized
		);
	});
}

#[test]
fn it_processes_a_invalid_header_update_with_duplicate_entry() {
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();
	let update = get_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
	let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
		update.attested_header.slot,
	);

	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(&initial_sync));

		LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
			beacon_block_root: H256::default(),
			beacon_slot: update.attested_header.slot,
			import_time: 0,
		});

		assert_ok!(mock_minimal::EthereumBeaconClient::store_sync_committee(
			current_period,
			&initial_sync.current_sync_committee,
		));

		LatestExecutionHeader::<mock_minimal::Test>::set(ExecutionHeaderState {
			beacon_block_root: Default::default(),
			beacon_slot: 0,
			block_hash: Default::default(),
			// initialize with the same block_number in execution_payload of the next update
			block_number: update.execution_header.block_number,
		});

		assert_err!(
			mock_minimal::EthereumBeaconClient::import_execution_header(
				mock_minimal::RuntimeOrigin::signed(1),
				update
			),
			Error::<mock_minimal::Test>::InvalidExecutionHeaderUpdate
		);
	});
}

#[test]
fn it_errors_when_importing_a_header_with_no_sync_committee_for_period() {
	let update = get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

	new_tester::<mock_minimal::Test>().execute_with(|| {
		ValidatorsRoot::<mock_minimal::Test>::set(
			hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into(),
		);

		LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
			beacon_block_root: H256::default(),
			beacon_slot: update.finalized_header.slot - 1,
			import_time: 0,
		});

		assert_err!(
			mock_minimal::EthereumBeaconClient::import_finalized_header(
				mock_minimal::RuntimeOrigin::signed(1),
				update
			),
			Error::<mock_minimal::Test>::SyncCommitteeMissing
		);
	});
}

#[test]
pub fn test_hash_tree_root_sync_committee() {
	let sync_committee = get_committee_sync_ssz_test_data::<SYNC_COMMITTEE_SIZE>();
	let hash_root_result = sync_committee.hash_tree_root();
	assert_ok!(&hash_root_result);

	let hash_root: H256 = hash_root_result.unwrap().into();
	assert_eq!(
		hash_root,
		hex!("7ba44032b68620539b1bac45e5202dd530af5f6b669a5a496ba0fcfb3f0b8da3").into()
	);
}

#[test]
pub fn test_bls_fast_aggregate_verify() {
	let test_data =
		get_bls_signature_verify_test_data::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

	let milagro_pubkeys = prepare_g1_pubkeys(&test_data.pubkeys.to_vec()).unwrap();

	let participant_bits = decompress_sync_committee_bits::<
		SYNC_COMMITTEE_SIZE,
		SYNC_COMMITTEE_BITS_SIZE,
	>(test_data.sync_committee_bits);

	let participant_pubkeys =
		mock_minimal::EthereumBeaconClient::find_pubkeys(&participant_bits, &milagro_pubkeys, true);

	let signing_root = mock_minimal::EthereumBeaconClient::signing_root(
		&test_data.header,
		test_data.validators_root,
		test_data.signature_slot,
	)
	.unwrap();

	assert_ok!(fast_aggregate_verify_legacy(
		&participant_pubkeys,
		signing_root,
		&test_data.sync_committee_signature,
	));
}
