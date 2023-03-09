#[cfg(not(feature = "minimal"))]
mod beacon_mainnet_tests {
	use crate::{
		config, merkleization, mock::*, Error, ExecutionHeaders, FinalizedBeaconHeaders,
		FinalizedBeaconHeadersBlockRoot, FinalizedHeaderState, LatestFinalizedHeaderState,
		LatestSyncCommitteePeriod, SyncCommittees, ValidatorsRoot,
	};
	use frame_support::{assert_err, assert_ok};
	use hex_literal::hex;
	use sp_core::H256;

	#[test]
	fn it_syncs_from_an_initial_checkpoint() {
		let initial_sync = get_initial_sync::<mock_mainnet::Test>();

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			assert_ok!(mock_mainnet::EthereumBeaconClient::initial_sync(initial_sync.clone()));

			let block_root: H256 =
				merkleization::hash_tree_root_beacon_header(initial_sync.header.clone())
					.unwrap()
					.into();

			assert!(<FinalizedBeaconHeaders<mock_mainnet::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_updates_a_committee_period_sync_update() {
		let update = get_committee_sync_period_update::<mock_mainnet::Test>();

		let current_sync_committee =
			get_initial_sync::<mock_mainnet::Test>().current_sync_committee;

		let current_period = mock_mainnet::EthereumBeaconClient::compute_current_sync_period(
			update.attested_header.slot,
		);

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			SyncCommittees::<mock_mainnet::Test>::insert(current_period, current_sync_committee);
			LatestSyncCommitteePeriod::<mock_mainnet::Test>::set(current_period);
			ValidatorsRoot::<mock_mainnet::Test>::set(get_validators_root::<mock_mainnet::Test>());

			assert_ok!(mock_mainnet::EthereumBeaconClient::sync_committee_period_update(
				mock_mainnet::RuntimeOrigin::signed(1),
				update.clone(),
			));

			let block_root: H256 =
				merkleization::hash_tree_root_beacon_header(update.finalized_header.clone())
					.unwrap()
					.into();

			assert!(<FinalizedBeaconHeaders<mock_mainnet::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_processes_a_finalized_header_update() {
		let update = get_finalized_header_update::<mock_mainnet::Test>();
		let initial_sync = get_initial_sync::<mock_mainnet::Test>();
		let current_sync_committee = initial_sync.current_sync_committee;

		let current_period = mock_mainnet::EthereumBeaconClient::compute_current_sync_period(
			update.attested_header.slot,
		);

		let slot = initial_sync.header.slot;
		let import_time = 1616508000u64 + (slot * config::SECONDS_PER_SLOT); // Goerli genesis time + finalized header update time
		let mock_pallet_time = import_time + 3600; // plus one hour

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			mock_mainnet::Timestamp::set_timestamp(mock_pallet_time * 1000); // needs to be in milliseconds
			SyncCommittees::<mock_mainnet::Test>::insert(current_period, current_sync_committee);
			LatestFinalizedHeaderState::<mock_mainnet::Test>::set(FinalizedHeaderState {
				beacon_block_root: Default::default(),
				import_time,
				beacon_slot: slot - 1,
			});
			ValidatorsRoot::<mock_mainnet::Test>::set(get_validators_root::<mock_mainnet::Test>());

			assert_ok!(mock_mainnet::EthereumBeaconClient::import_finalized_header(
				mock_mainnet::RuntimeOrigin::signed(1),
				update.clone()
			));

			let block_root: H256 =
				merkleization::hash_tree_root_beacon_header(update.finalized_header.clone())
					.unwrap()
					.into();

			assert!(<FinalizedBeaconHeaders<mock_mainnet::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_errors_when_weak_subjectivity_period_exceeded_for_a_finalized_header_update() {
		let update = get_finalized_header_update::<mock_mainnet::Test>();
		let initial_sync = get_initial_sync::<mock_mainnet::Test>();
		let current_sync_committee = initial_sync.current_sync_committee;

		let current_period = mock_mainnet::EthereumBeaconClient::compute_current_sync_period(
			update.attested_header.slot,
		);

		let slot = initial_sync.header.slot;
		let import_time = 1616508000u64 + (slot * config::SECONDS_PER_SLOT);
		let mock_pallet_time = import_time + 100800; // plus 28 hours

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			mock_mainnet::Timestamp::set_timestamp(mock_pallet_time * 1000); // needs to be in milliseconds
			SyncCommittees::<mock_mainnet::Test>::insert(current_period, current_sync_committee);
			LatestFinalizedHeaderState::<mock_mainnet::Test>::set(FinalizedHeaderState {
				beacon_block_root: Default::default(),
				import_time,
				beacon_slot: slot - 1,
			});
			ValidatorsRoot::<mock_mainnet::Test>::set(get_validators_root::<mock_mainnet::Test>());

			assert_err!(
				mock_mainnet::EthereumBeaconClient::import_finalized_header(
					mock_mainnet::RuntimeOrigin::signed(1),
					update.clone()
				),
				Error::<mock_mainnet::Test>::BridgeBlocked
			);
		});
	}

	#[test]
	fn it_processes_a_header_update() {
		let update = get_header_update::<mock_mainnet::Test>();

		let current_sync_committee =
			get_initial_sync::<mock_mainnet::Test>().current_sync_committee;

		let current_period = mock_mainnet::EthereumBeaconClient::compute_current_sync_period(
			update.beacon_header.slot,
		);

		let finalized_update = get_finalized_header_update::<mock_mainnet::Test>();
		let finalized_slot = finalized_update.finalized_header.slot;
		let finalized_block_root: H256 =
			merkleization::hash_tree_root_beacon_header(finalized_update.finalized_header)
				.unwrap()
				.into();

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			SyncCommittees::<mock_mainnet::Test>::insert(current_period, current_sync_committee);
			ValidatorsRoot::<mock_mainnet::Test>::set(get_validators_root::<mock_mainnet::Test>());
			LatestFinalizedHeaderState::<mock_mainnet::Test>::set(FinalizedHeaderState {
				beacon_block_root: finalized_block_root,
				beacon_slot: finalized_slot,
				import_time: 0,
			});
			FinalizedBeaconHeadersBlockRoot::<mock_mainnet::Test>::insert(
				finalized_block_root,
				finalized_update.block_roots_hash,
			);

			assert_ok!(mock_mainnet::EthereumBeaconClient::import_execution_header(
				mock_mainnet::RuntimeOrigin::signed(1),
				update.clone()
			));

			let execution_block_root: H256 = update.execution_header.block_hash.clone().into();

			assert!(<ExecutionHeaders<mock_mainnet::Test>>::contains_key(execution_block_root));
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
			hex!("99daf976424b62249669bc842e9b8e5a5a2960d1d81d98c3267f471409c3c841").into()
		);
	}

	#[test]
	pub fn test_bls_fast_aggregate_verify() {
		let test_data = get_bls_signature_verify_test_data::<mock_mainnet::Test>();

		let sync_committee_bits =
			merkleization::get_sync_committee_bits::<mock_mainnet::MaxSyncCommitteeSize>(
				test_data.sync_committee_bits.try_into().expect("too many sync committee bits"),
			);

		assert_ok!(&sync_committee_bits);

		assert_ok!(mock_mainnet::EthereumBeaconClient::verify_signed_header(
			sync_committee_bits.unwrap(),
			test_data.sync_committee_signature.try_into().expect("signature is too long"),
			test_data.pubkeys.to_vec().try_into().expect("to many pubkeys"),
			test_data.header,
			test_data.validators_root,
			test_data.signature_slot,
		));
	}
}
