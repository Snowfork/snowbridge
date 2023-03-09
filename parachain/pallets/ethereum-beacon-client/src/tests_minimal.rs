#[cfg(feature = "minimal")]
mod beacon_minimal_tests {
	use crate::{
		config, merkleization, merkleization::MerkleizationError, mock::*,
		pallet::FinalizedBeaconHeadersBlockRoot, ssz::SSZBeaconBlockBody, Error,
		ExecutionHeaderState, ExecutionHeaders, FinalizedBeaconHeaders, FinalizedHeaderState,
		LatestExecutionHeaderState, LatestFinalizedHeaderState, LatestSyncCommitteePeriod,
		SyncCommittees, ValidatorsRoot,
	};
	use frame_support::{assert_err, assert_ok};
	use hex_literal::hex;
	use sp_core::H256;
	use std::time::{SystemTime, UNIX_EPOCH};

	#[test]
	fn it_syncs_from_an_initial_checkpoint() {
		let initial_sync = get_initial_sync::<mock_minimal::Test>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::initial_sync(initial_sync.clone()));

			let block_root: H256 =
				merkleization::hash_tree_root_beacon_header(initial_sync.header.clone())
					.unwrap()
					.into();

			assert!(<FinalizedBeaconHeaders<mock_minimal::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_updates_a_committee_period_sync_update() {
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

			LatestSyncCommitteePeriod::<mock_minimal::Test>::set(current_period);

			assert_ok!(mock_minimal::EthereumBeaconClient::sync_committee_period_update(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone(),
			));

			let block_root: H256 =
				merkleization::hash_tree_root_beacon_header(update.finalized_header.clone())
					.unwrap()
					.into();

			assert!(<FinalizedBeaconHeaders<mock_minimal::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_updates_a_committee_period_sync_update_with_invalid_header() {
		let initial_sync = get_initial_sync::<mock_minimal::Test>();

		let mut update = get_committee_sync_period_update::<mock_minimal::Test>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::initial_sync(initial_sync.clone()));

			// makes a invalid update with signature_slot should be more than attested_slot
			update.signature_slot = update.attested_header.slot;

			assert_err!(
				mock_minimal::EthereumBeaconClient::sync_committee_period_update(
					mock_minimal::RuntimeOrigin::signed(1),
					update.clone(),
				),
				Error::<mock_minimal::Test>::InvalidSyncCommitteeHeaderUpdate
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
				Error::<mock_minimal::Test>::InvalidSyncCommitteePeriodUpdateWithGap
			);
		});
	}*/

	#[test]
	fn it_updates_a_invalid_committee_period_sync_update_with_duplicate_entry() {
		let initial_sync = get_initial_sync::<mock_minimal::Test>();

		let update = get_committee_sync_period_update::<mock_minimal::Test>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::initial_sync(initial_sync.clone()));

			let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
				update.attested_header.slot,
			);

			SyncCommittees::<mock_minimal::Test>::insert(
				current_period,
				initial_sync.current_sync_committee.clone(),
			);

			// initialize with period of the next update
			SyncCommittees::<mock_minimal::Test>::insert(
				current_period + 1,
				initial_sync.current_sync_committee,
			);

			LatestSyncCommitteePeriod::<mock_minimal::Test>::set(current_period + 1);

			assert_err!(
				mock_minimal::EthereumBeaconClient::sync_committee_period_update(
					mock_minimal::RuntimeOrigin::signed(1),
					update.clone(),
				),
				Error::<mock_minimal::Test>::InvalidSyncCommitteePeriodUpdateWithDuplication
			);
		});
	}

	#[test]
	fn it_processes_a_finalized_header_update() {
		let update = get_finalized_header_update::<mock_minimal::Test>();
		let initial_sync = get_initial_sync::<mock_minimal::Test>();
		let current_sync_committee = initial_sync.current_sync_committee;

		let current_period = mock_minimal::EthereumBeaconClient::compute_current_sync_period(
			update.attested_header.slot,
		);

		let time_now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards")
			.as_secs();

		let import_time = time_now + (update.finalized_header.slot * config::SECONDS_PER_SLOT); // Goerli genesis time + finalized header update time
		let mock_pallet_time = import_time + 3600; // plus one hour

		new_tester::<mock_minimal::Test>().execute_with(|| {
			mock_minimal::Timestamp::set_timestamp(mock_pallet_time * 1000); // needs to be in milliseconds
			LatestFinalizedHeaderState::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: Default::default(),
				import_time,
				// set the last imported finalized header to an older finalized header. Necessary
				// for long range attack check and finalized header to be imported must not have
				// been imported already.
				beacon_slot: update.finalized_header.slot - 1,
			});
			SyncCommittees::<mock_minimal::Test>::insert(current_period, current_sync_committee);
			ValidatorsRoot::<mock_minimal::Test>::set(get_validators_root::<mock_minimal::Test>());

			assert_ok!(mock_minimal::EthereumBeaconClient::import_finalized_header(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone()
			));

			let block_root: H256 =
				merkleization::hash_tree_root_beacon_header(update.finalized_header.clone())
					.unwrap()
					.into();

			assert!(<FinalizedBeaconHeaders<mock_minimal::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_processes_a_invalid_finalized_header_update() {
		let update = get_finalized_header_update::<mock_minimal::Test>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			LatestFinalizedHeaderState::<mock_minimal::Test>::set(FinalizedHeaderState {
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
				Error::<mock_minimal::Test>::InvalidFinalizedHeaderUpdate
			);
		});
	}

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
	}

	#[test]
	fn it_processes_a_header_update() {
		let update = get_header_update::<mock_minimal::Test>();

		let current_sync_committee =
			get_initial_sync::<mock_minimal::Test>().current_sync_committee;

		let finalized_update = get_finalized_header_update::<mock_minimal::Test>();
		let finalized_slot = finalized_update.finalized_header.slot;
		let finalized_block_root: H256 =
			merkleization::hash_tree_root_beacon_header(finalized_update.finalized_header)
				.unwrap()
				.into();

		let current_period =
			mock_minimal::EthereumBeaconClient::compute_current_sync_period(update.block.slot);

		new_tester::<mock_minimal::Test>().execute_with(|| {
			SyncCommittees::<mock_minimal::Test>::insert(current_period, current_sync_committee);
			ValidatorsRoot::<mock_minimal::Test>::set(get_validators_root::<mock_minimal::Test>());
			LatestFinalizedHeaderState::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: finalized_block_root,
				beacon_slot: finalized_slot,
				import_time: 0,
			});
			FinalizedBeaconHeadersBlockRoot::<mock_minimal::Test>::insert(
				finalized_block_root,
				finalized_update.block_roots_hash,
			);

			assert_ok!(mock_minimal::EthereumBeaconClient::import_execution_header(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone()
			));

			let execution_block_root: H256 =
				update.block.body.execution_payload.block_hash.clone().into();

			assert!(<ExecutionHeaders<mock_minimal::Test>>::contains_key(execution_block_root));
		});
	}

	#[test]
	fn it_processes_a_invalid_header_update_not_finalized() {
		let update = get_header_update::<mock_minimal::Test>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			LatestFinalizedHeaderState::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: H256::default(),
				// initialize finalized state with parent slot of the next update
				beacon_slot: update.block.slot - 1,
				import_time: 0,
			});

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
		let update = get_header_update::<mock_minimal::Test>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			LatestFinalizedHeaderState::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: H256::default(),
				beacon_slot: update.block.slot,
				import_time: 0,
			});

			LatestExecutionHeaderState::<mock_minimal::Test>::set(ExecutionHeaderState {
				beacon_block_root: Default::default(),
				beacon_slot: 0,
				block_hash: Default::default(),
				// initialize with the same block_number in execution_payload of the next update
				block_number: update.block.body.execution_payload.block_number,
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
		let update = get_finalized_header_update::<mock_minimal::Test>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			ValidatorsRoot::<mock_minimal::Test>::set(
				hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into(),
			);

			LatestFinalizedHeaderState::<mock_minimal::Test>::set(FinalizedHeaderState {
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
		let sync_committee = get_committee_sync_ssz_test_data::<mock_mainnet::Test>();
		let hash_root_result = merkleization::hash_tree_root_sync_committee(sync_committee);
		assert_ok!(&hash_root_result);

		let hash_root: H256 = hash_root_result.unwrap().into();
		assert_eq!(
			hash_root,
			hex!("7ba44032b68620539b1bac45e5202dd530af5f6b669a5a496ba0fcfb3f0b8da3").into()
		);
	}

	#[test]
	pub fn test_bls_fast_aggregate_verify() {
		let test_data = get_bls_signature_verify_test_data::<mock_minimal::Test>();

		let sync_committee_bits =
			merkleization::get_sync_committee_bits::<mock_minimal::MaxSyncCommitteeSize>(
				test_data.sync_committee_bits.try_into().expect("too many sync committee bits"),
			);

		assert_ok!(&sync_committee_bits);

		assert_ok!(mock_minimal::EthereumBeaconClient::verify_signed_header(
			sync_committee_bits.unwrap(),
			test_data.sync_committee_signature.try_into().expect("signature is too long"),
			test_data.pubkeys.to_vec().try_into().expect("to many pubkeys"),
			test_data.header,
			test_data.validators_root,
			test_data.signature_slot,
		));
	}
}
