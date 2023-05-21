use crate::{
	mock::{mock_minimal, new_tester},
	pallet::ExecutionHeaders,
	sync_committee_sum, verify_merkle_branch, BeaconHeader, Error, H256,
};
use frame_support::{assert_err, assert_ok};
use hex_literal::hex;
use primitives::{
	fast_aggregate_verify_legacy, prepare_g1_pubkeys, BlsError, CompactExecutionHeader, PublicKey,
	PublicKeyPrepared,
};
use rand::{thread_rng, Rng};

pub fn prepare_milagro_pubkeys() -> Result<Vec<PublicKeyPrepared>, &'static str> {
	let pubkeys: Vec<PublicKey> = vec![
		hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into(),
		hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into(),
		hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into(),
		hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into(),
	];
	let milagro_pubkeys = prepare_g1_pubkeys(&pubkeys).unwrap();
	Ok(milagro_pubkeys)
}

#[test]
pub fn test_sync_committee_sum() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_eq!(sync_committee_sum(&[0, 1, 0, 1, 1, 0, 1, 0, 1]), 5);
	});
}

#[test]
pub fn test_compute_domain() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let domain = mock_minimal::EthereumBeaconClient::compute_domain(
			hex!("07000000").into(),
			hex!("00000001").into(),
			hex!("5dec7ae03261fde20d5b024dfabce8bac3276c9a4908e23d50ba8c9b50b0adff").into(),
		);

		assert_ok!(&domain);
		assert_eq!(
			domain.unwrap(),
			hex!("0700000046324489ceb6ada6d118eacdbe94f49b1fcb49d5481a685979670c7c").into()
		);
	});
}

#[test]
pub fn test_compute_domain_kiln() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let domain = mock_minimal::EthereumBeaconClient::compute_domain(
			hex!("07000000").into(),
			hex!("70000071").into(),
			hex!("99b09fcd43e5905236c370f184056bec6e6638cfc31a323b304fc4aa789cb4ad").into(),
		);

		assert_ok!(&domain);
		assert_eq!(
			domain.unwrap(),
			hex!("07000000e7acb21061790987fa1c1e745cccfb358370b33e8af2b2c18938e6c2").into()
		);
	});
}

#[test]
pub fn test_compute_signing_root_bls() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let signing_root = mock_minimal::EthereumBeaconClient::compute_signing_root(
			&BeaconHeader {
				slot: 3529537,
				proposer_index: 192549,
				parent_root: hex!(
					"1f8dc05ea427f78e84e2e2666e13c3befb7106fd1d40ef8a3f67cf615f3f2a4c"
				)
				.into(),
				state_root: hex!(
					"0dfb492a83da711996d2d76b64604f9bca9dc08b6c13cf63b3be91742afe724b"
				)
				.into(),
				body_root: hex!("66fba38f7c8c2526f7ddfe09c1a54dd12ff93bdd4d0df6a0950e88e802228bfa")
					.into(),
			},
			hex!("07000000afcaaba0efab1ca832a15152469bb09bb84641c405171dfa2d3fb45f").into(),
		);

		assert_ok!(&signing_root);
		assert_eq!(
			signing_root.unwrap(),
			hex!("3ff6e9807da70b2f65cdd58ea1b25ed441a1d589025d2c4091182026d7af08fb").into()
		);
	});
}

#[test]
pub fn test_compute_signing_root_kiln() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let signing_root = mock_minimal::EthereumBeaconClient::compute_signing_root(
			&BeaconHeader {
				slot: 221316,
				proposer_index: 79088,
				parent_root: hex!(
					"b4c15cd79da1a4e645b0104fa66d226cb6dce0fae3522789cc4d0b3ae41d96f7"
				)
				.into(),
				state_root: hex!(
					"6f711ef2e36decbc8f7037e73bbdace42c11f2896a43e44ab8a78dcb2ba66122"
				)
				.into(),
				body_root: hex!("963eaa01341c16dc8f288da47eedad0792978fdaab9f1f97ae0a1103494d1a10")
					.into(),
			},
			hex!("07000000afcaaba0efab1ca832a15152469bb09bb84641c405171dfa2d3fb45f").into(),
		);

		assert_ok!(&signing_root);
		assert_eq!(
			signing_root.unwrap(),
			hex!("4ce7b4192c0292a2bbf4107766ddc0f613261bb8e6968ccd0e6b71b30fad6d7c").into()
		);
	});
}

#[test]
pub fn test_compute_signing_root_kiln_head_update() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let signing_root = mock_minimal::EthereumBeaconClient::compute_signing_root(
			&BeaconHeader {
				slot: 222472,
				proposer_index: 10726,
				parent_root: hex!(
					"5d481a9721f0ecce9610eab51d400d223683d599b7fcebca7e4c4d10cdef6ebb"
				)
				.into(),
				state_root: hex!(
					"14eb4575895f996a84528b789ff2e4d5148242e2983f03068353b2c37015507a"
				)
				.into(),
				body_root: hex!("7bb669c75b12e0781d6fa85d7fc2f32d64eafba89f39678815b084c156e46cac")
					.into(),
			},
			hex!("07000000e7acb21061790987fa1c1e745cccfb358370b33e8af2b2c18938e6c2").into(),
		);

		assert_ok!(&signing_root);
		assert_eq!(
			signing_root.unwrap(),
			hex!("da12b6a6d3516bc891e8a49f82fc1925cec40b9327e06457f695035303f55cd8").into()
		);
	});
}

#[test]
pub fn test_compute_domain_bls() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let domain = mock_minimal::EthereumBeaconClient::compute_domain(
			hex!("07000000").into(),
			hex!("01000000").into(),
			hex!("4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95").into(),
		);

		assert_ok!(&domain);
		assert_eq!(
			domain.unwrap(),
			hex!("07000000afcaaba0efab1ca832a15152469bb09bb84641c405171dfa2d3fb45f").into()
		);
	});
}

#[test]
pub fn test_is_valid_merkle_proof() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_eq!(
			verify_merkle_branch(
				hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
				&[
					hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
					hex!("5f6f02af29218292d21a69b64a794a7c0873b3e0f54611972863706e8cbdf371").into(),
					hex!("e7125ff9ab5a840c44bedb4731f440a405b44e15f2d1a89e27341b432fabe13d").into(),
					hex!("002c1fe5bc0bd62db6f299a582f2a80a6d5748ccc82e7ed843eaf0ae0739f74a").into(),
					hex!("d2dc4ba9fd4edff6716984136831e70a6b2e74fca27b8097a820cbbaa5a6e3c3").into(),
					hex!("91f77a19d8afa4a08e81164bb2e570ecd10477b3b65c305566a6d2be88510584").into(),
				],
				crate::config::FINALIZED_ROOT_INDEX,
				crate::config::FINALIZED_ROOT_DEPTH,
				hex!("e46559327592741956f6beaa0f52e49625eb85dce037a0bd2eff333c743b287f").into()
			)
			.is_some_and(|x| x),
			true
		);
	});
}

#[test]
pub fn test_merkle_proof_fails_if_depth_and_branch_dont_match() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		assert_eq!(
			verify_merkle_branch(
				hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
				&[
					hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
					hex!("5f6f02af29218292d21a69b64a794a7c0873b3e0f54611972863706e8cbdf371").into(),
					hex!("e7125ff9ab5a840c44bedb4731f440a405b44e15f2d1a89e27341b432fabe13d").into(),
				],
				crate::config::FINALIZED_ROOT_INDEX,
				crate::config::FINALIZED_ROOT_DEPTH,
				hex!("e46559327592741956f6beaa0f52e49625eb85dce037a0bd2eff333c743b287f").into()
			)
			.is_some_and(|x| x),
			false
		);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_minimal() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let milagro_pubkeys = prepare_milagro_pubkeys().unwrap();
		assert_ok!(fast_aggregate_verify_legacy(
			&milagro_pubkeys,
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			&hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").into()
		));
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_point() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let pubkeys: Vec<PublicKey> = vec![
			hex!("973eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into(),
			hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into(),
			hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into(),
			hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into(),
		];
		assert_err!(prepare_g1_pubkeys(&pubkeys), BlsError::InvalidPublicKey);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_message() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let milagro_pubkeys = prepare_milagro_pubkeys().unwrap();
		assert_err!(fast_aggregate_verify_legacy(
			&milagro_pubkeys,
			hex!("99241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			&hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").into()
		), BlsError::SignatureVerificationFailed);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_signature() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let milagro_pubkeys = prepare_milagro_pubkeys().unwrap();
		assert_err!(fast_aggregate_verify_legacy(
			&milagro_pubkeys,
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			&hex!("c204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").into()
		), BlsError::InvalidSignature);
	});
}

#[test]
pub fn test_sync_committee_participation_is_supermajority() {
	let bits =
hex!("bffffffff7f1ffdfcfeffeffbfdffffbfffffdffffefefffdffff7f7ffff77fffdf7bff77ffdf7fffafffffff77fefffeff7effffffff5f7fedfffdfb6ddff7b"
);
	let participation = primitives::decompress_sync_committee_bits::<512, 64>(bits);
	assert_ok!(mock_minimal::EthereumBeaconClient::sync_committee_participation_is_supermajority(
		&participation
	));
}

#[test]
pub fn test_sync_committee_participation_is_supermajority_errors_when_not_supermajority() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let participation: [u8; 512] = [
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
			1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0,
			1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
			1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1,
			0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1,
			1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
		];

		assert_err!(
			mock_minimal::EthereumBeaconClient::sync_committee_participation_is_supermajority(
				&participation
			),
			Error::<mock_minimal::Test>::SyncCommitteeParticipantsNotSupermajority
		);
	});
}

// #[test]
// pub fn test_prune_finalized_header() {
// 	new_tester::<mock_minimal::Test>().execute_with(|| {
// 		let max_finalized_slots =
// 			mock_minimal::FinalizedHeaderPruneThreshold::get().try_into().unwrap();

// 		// Keeping track of to be deleted data
// 		let amount_of_data_to_be_deleted = max_finalized_slots / 2;
// 		let mut to_be_deleted_hash_list = vec![];
// 		let mut to_be_preserved_hash_list = vec![];
// 		for i in 0..max_finalized_slots {
// 			let mut hash = H256::default();
// 			thread_rng().try_fill(&mut hash.0[..]).unwrap();

// 			if i < amount_of_data_to_be_deleted {
// 				to_be_deleted_hash_list.push(hash);
// 			} else {
// 				to_be_preserved_hash_list.push(hash);
// 			}
// 			let finalized_state = FinalizedHeaderState {
// 				beacon_block_root: hash,
// 				beacon_slot: i,
// 				import_time: u64::default(),
// 			};

// 			FinalizedBeaconHeadersBlockRoot::<mock_minimal::Test>::insert(hash, hash);
// 			FinalizedBeaconHeaders::<mock_minimal::Test>::insert(hash, BeaconHeader::default());
// 			assert_ok!(mock_minimal::EthereumBeaconClient::add_finalized_header_state(
// 				finalized_state
// 			));
// 		}

// 		// We first verify if the data corresponding to that hash is still there.
// 		let slot_vec = FinalizedBeaconHeaderStates::<mock_minimal::Test>::get();
// 		assert_eq!(slot_vec.len(), max_finalized_slots as usize);
// 		for i in 0..(amount_of_data_to_be_deleted as usize) {
// 			assert_eq!(slot_vec[i].beacon_slot, i as u64);
// 			assert_eq!(slot_vec[i].beacon_block_root, to_be_deleted_hash_list[i]);

// 			assert!(FinalizedBeaconHeadersBlockRoot::<mock_minimal::Test>::contains_key(
// 				to_be_deleted_hash_list[i]
// 			));
// 			assert!(FinalizedBeaconHeaders::<mock_minimal::Test>::contains_key(
// 				to_be_deleted_hash_list[i]
// 			));
// 		}

// 		// We insert `amount_of_hash_to_be_deleted` number of new finalized headers
// 		for i in max_finalized_slots..(max_finalized_slots + amount_of_data_to_be_deleted) {
// 			let mut hash = H256::default();
// 			thread_rng().try_fill(&mut hash.0[..]).unwrap();
// 			FinalizedBeaconHeadersBlockRoot::<mock_minimal::Test>::insert(hash, hash);
// 			FinalizedBeaconHeaders::<mock_minimal::Test>::insert(hash, BeaconHeader::default());
// 			let finalized_state = FinalizedHeaderState {
// 				beacon_block_root: hash,
// 				beacon_slot: i,
// 				import_time: u64::default(),
// 			};
// 			assert_ok!(mock_minimal::EthereumBeaconClient::add_finalized_header_state(
// 				finalized_state
// 			));
// 		}

// 		// Now, previous hashes should be pruned and in array those elements are replaced by later
// 		// elements
// 		let slot_vec = FinalizedBeaconHeaderStates::<mock_minimal::Test>::get();
// 		assert_eq!(slot_vec.len(), max_finalized_slots as usize);
// 		for i in 0..(amount_of_data_to_be_deleted as usize) {
// 			assert_eq!(slot_vec[i].beacon_slot, (i as u64 + amount_of_data_to_be_deleted));
// 			assert_eq!(slot_vec[i].beacon_block_root, to_be_preserved_hash_list[i]);

// 			// Previous values should not exists
// 			assert!(!FinalizedBeaconHeadersBlockRoot::<mock_minimal::Test>::contains_key(
// 				to_be_deleted_hash_list[i]
// 			));
// 			assert!(!FinalizedBeaconHeaders::<mock_minimal::Test>::contains_key(
// 				to_be_deleted_hash_list[i]
// 			));

// 			// data that was preserved should exists
// 			assert!(FinalizedBeaconHeadersBlockRoot::<mock_minimal::Test>::contains_key(
// 				to_be_preserved_hash_list[i]
// 			));
// 			assert!(FinalizedBeaconHeaders::<mock_minimal::Test>::contains_key(
// 				to_be_preserved_hash_list[i]
// 			));
// 		}
// 	});
// }

#[test]
pub fn test_prune_execution_headers() {
	new_tester::<mock_minimal::Test>().execute_with(|| {
		let execution_header_prune_threshold = mock_minimal::ExecutionHeadersPruneThreshold::get();
		let to_be_deleted = execution_header_prune_threshold / 2;

		let mut stored_hashes = vec![];

		for i in 0..execution_header_prune_threshold {
			let mut hash = H256::default();
			thread_rng().try_fill(&mut hash.0[..]).unwrap();
			mock_minimal::EthereumBeaconClient::store_execution_header(
				hash,
				CompactExecutionHeader::default(),
				i as u64,
				hash,
			);
			stored_hashes.push(hash);
		}

		// We should have stored everything until now
		assert_eq!(
			ExecutionHeaders::<mock_minimal::Test>::iter().count() as usize,
			stored_hashes.len()
		);

		// Let's push extra entries so that some of the previous entries are deleted.
		for i in 0..to_be_deleted {
			let mut hash = H256::default();
			thread_rng().try_fill(&mut hash.0[..]).unwrap();
			mock_minimal::EthereumBeaconClient::store_execution_header(
				hash,
				CompactExecutionHeader::default(),
				(i + execution_header_prune_threshold) as u64,
				hash,
			);

			stored_hashes.push(hash);
		}

		// We should have only stored upto `execution_header_prune_threshold`
		assert_eq!(
			ExecutionHeaders::<mock_minimal::Test>::iter().count() as u32,
			execution_header_prune_threshold
		);

		// First `to_be_deleted` items must be deleted
		for i in 0..to_be_deleted {
			assert!(!ExecutionHeaders::<mock_minimal::Test>::contains_key(
				stored_hashes[i as usize]
			));
		}

		// Other entries should be part of data
		for i in to_be_deleted..(to_be_deleted + execution_header_prune_threshold) {
			assert!(ExecutionHeaders::<mock_minimal::Test>::contains_key(
				stored_hashes[i as usize]
			));
		}
	});
}

//#[cfg(feature = "minimal")]
mod minimal_spec {
	use crate::{
		config::{SYNC_COMMITTEE_BITS_SIZE, SYNC_COMMITTEE_SIZE},
		mock::*,
		CompactBeaconState, CurrentSyncCommittee, Error, ExecutionHeaderState, ExecutionHeaders,
		FinalizedBeaconState, FinalizedHeaderState, LatestExecutionHeader, LatestFinalizedHeader,
		NextSyncCommittee, SyncCommitteePrepared, ValidatorsRoot,
	};
	use frame_support::{assert_err, assert_ok};
	use primitives::{
		decompress_sync_committee_bits, fast_aggregate_verify_legacy, prepare_g1_pubkeys,
	};
	use sp_core::H256;

	#[test]
	fn it_syncs_from_an_initial_checkpoint() {
		let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(
				&initial_sync
			));

			let block_root: H256 = initial_sync.header.hash_tree_root().unwrap();

			assert!(<FinalizedBeaconState<mock_minimal::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_updates_sync_committee() {
		let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

		let update =
			get_committee_sync_period_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(
				&initial_sync
			));

			let sync_committee_prepared: SyncCommitteePrepared =
				(&initial_sync.current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			assert!(!<NextSyncCommittee<mock_minimal::Test>>::exists());

			assert_ok!(mock_minimal::EthereumBeaconClient::submit(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone(),
			));

			assert!(<NextSyncCommittee<mock_minimal::Test>>::exists());
		});
	}

	#[test]
	fn it_updates_a_committee_period_sync_update_with_invalid_signature_slot() {
		let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

		let mut update =
			get_committee_sync_period_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(
				&initial_sync
			));

			// makes a invalid update with signature_slot should be more than attested_slot
			update.signature_slot = update.attested_header.slot;

			assert_err!(
				mock_minimal::EthereumBeaconClient::submit(
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

			let current_period = compute_period(
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
			assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(
				&initial_sync
			));

			let sync_committee_prepared: SyncCommitteePrepared =
				(&initial_sync.current_sync_committee).try_into().unwrap();
			<NextSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			assert_err!(
				mock_minimal::EthereumBeaconClient::submit(
					mock_minimal::RuntimeOrigin::signed(1),
					update.clone(),
				),
				Error::<mock_minimal::Test>::NotRelevant
			);
		});
	}

	#[test]
	fn it_processes_a_finalized_header_update() {
		let update = get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
		let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: Default::default(),
				// set the last imported finalized header to an older finalized header. Necessary
				// for long range attack check and finalized header to be imported must not have
				// been imported already.
				beacon_slot: update.finalized_header.slot - 1,
			});
			ValidatorsRoot::<mock_minimal::Test>::set(get_validators_root::<SYNC_COMMITTEE_SIZE>());
			let sync_committee_prepared: SyncCommitteePrepared =
				(&initial_sync.current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			assert_ok!(mock_minimal::EthereumBeaconClient::submit(
				mock_minimal::RuntimeOrigin::signed(1),
				update.clone()
			));

			let block_root: H256 = update.finalized_header.clone().hash_tree_root().unwrap();

			assert!(<FinalizedBeaconState<mock_minimal::Test>>::contains_key(block_root));
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
		let update = get_header_update();

		let current_sync_committee =
			get_initial_sync::<SYNC_COMMITTEE_SIZE>().current_sync_committee;

		let finalized_update =
			get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
		let finalized_slot = finalized_update.finalized_header.slot;
		let finalized_block_root: H256 =
			finalized_update.finalized_header.hash_tree_root().unwrap();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			ValidatorsRoot::<mock_minimal::Test>::set(get_validators_root::<SYNC_COMMITTEE_SIZE>());
			LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: finalized_block_root,
				beacon_slot: finalized_slot,
			});
			FinalizedBeaconState::<mock_minimal::Test>::insert(
				finalized_block_root,
				CompactBeaconState {
					slot: finalized_update.finalized_header.slot,
					block_roots_root: finalized_update.block_roots_root,
				},
			);
			let sync_committee_prepared: SyncCommitteePrepared =
				(&current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			assert_ok!(mock_minimal::EthereumBeaconClient::submit_execution_header(
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
		let update = get_header_update();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(
				&initial_sync
			));

			LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: H256::default(),
				// initialize finalized state with parent slot of the next update
				beacon_slot: update.header.slot - 1,
			});

			let sync_committee_prepared: SyncCommitteePrepared =
				(&initial_sync.current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			assert_err!(
				mock_minimal::EthereumBeaconClient::submit_execution_header(
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
		let update = get_header_update();

		new_tester::<mock_minimal::Test>().execute_with(|| {
			assert_ok!(mock_minimal::EthereumBeaconClient::process_checkpoint_update(
				&initial_sync
			));

			LatestFinalizedHeader::<mock_minimal::Test>::set(FinalizedHeaderState {
				beacon_block_root: H256::default(),
				beacon_slot: update.header.slot,
			});

			let sync_committee_prepared: SyncCommitteePrepared =
				(&initial_sync.current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			LatestExecutionHeader::<mock_minimal::Test>::set(ExecutionHeaderState {
				beacon_block_root: Default::default(),
				beacon_slot: 0,
				block_hash: Default::default(),
				// initialize with the same block_number in execution_payload of the next update
				block_number: update.execution_header.block_number,
			});

			assert_err!(
				mock_minimal::EthereumBeaconClient::submit_execution_header(
					mock_minimal::RuntimeOrigin::signed(1),
					update
				),
				Error::<mock_minimal::Test>::InvalidExecutionHeaderUpdate
			);
		});
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

		let participant_pubkeys = mock_minimal::EthereumBeaconClient::find_pubkeys(
			&participant_bits,
			&milagro_pubkeys,
			true,
		);

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
}

#[cfg(not(feature = "minimal"))]
mod mainnet_spec {
	#[cfg(not(feature = "minimal"))]
	use crate::{
		compute_period, config,
		config::{SYNC_COMMITTEE_BITS_SIZE, SYNC_COMMITTEE_SIZE},
		mock::*,
		CompactBeaconState, CurrentSyncCommittee, ExecutionHeaders, FinalizedBeaconState,
		FinalizedHeaderState, LatestFinalizedHeader, SyncCommitteePrepared, ValidatorsRoot,
	};
	use frame_support::assert_ok;
	use primitives::{
		decompress_sync_committee_bits, fast_aggregate_verify_legacy, prepare_g1_pubkeys,
	};
	use sp_core::H256;

	#[test]
	fn it_syncs_from_an_initial_checkpoint() {
		let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			assert_ok!(mock_mainnet::EthereumBeaconClient::process_checkpoint_update(
				&initial_sync
			));

			let block_root: H256 = initial_sync.header.hash_tree_root().unwrap();

			assert!(<FinalizedBeaconState<mock_mainnet::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_updates_a_committee_period_sync_update() {
		let update =
			get_committee_sync_period_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
		let current_sync_committee =
			get_initial_sync::<{ SYNC_COMMITTEE_SIZE }>().current_sync_committee;
		let current_period = compute_period(update.attested_header.slot);

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			ValidatorsRoot::<mock_mainnet::Test>::set(
				get_validators_root::<{ SYNC_COMMITTEE_SIZE }>(),
			);

			let sync_committee_prepared: SyncCommitteePrepared =
				(&current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			let block_root: H256 = update.finalized_header.hash_tree_root().unwrap();

			assert_ok!(mock_mainnet::EthereumBeaconClient::submit(
				mock_mainnet::RuntimeOrigin::signed(1),
				update,
			));

			assert!(<FinalizedBeaconState<mock_mainnet::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_processes_a_finalized_header_update() {
		let update = get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
		let current_sync_committee =
			get_initial_sync::<{ SYNC_COMMITTEE_SIZE }>().current_sync_committee;
		let current_period = compute_period(update.attested_header.slot);

		let slot = update.finalized_header.slot;

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			let sync_committee_prepared: SyncCommitteePrepared =
				(&current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);

			LatestFinalizedHeader::<mock_mainnet::Test>::set(FinalizedHeaderState {
				beacon_block_root: Default::default(),
				beacon_slot: slot - 1,
			});
			ValidatorsRoot::<mock_mainnet::Test>::set(
				get_validators_root::<{ SYNC_COMMITTEE_SIZE }>(),
			);

			assert_ok!(mock_mainnet::EthereumBeaconClient::submit(
				mock_mainnet::RuntimeOrigin::signed(1),
				update.clone()
			));

			let block_root: H256 = update.finalized_header.hash_tree_root().unwrap();

			assert!(<FinalizedBeaconState<mock_mainnet::Test>>::contains_key(block_root));
		});
	}

	#[test]
	fn it_processes_a_header_update() {
		let update = get_header_update();
		let current_sync_committee =
			get_initial_sync::<{ config::SYNC_COMMITTEE_SIZE }>().current_sync_committee;
		let current_period = compute_period(update.header.slot);

		let finalized_update =
			get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
		let finalized_slot = finalized_update.finalized_header.slot;
		let finalized_block_root: H256 =
			finalized_update.finalized_header.hash_tree_root().unwrap();

		new_tester::<mock_mainnet::Test>().execute_with(|| {
			let sync_committee_prepared: SyncCommitteePrepared =
				(&current_sync_committee).try_into().unwrap();
			<CurrentSyncCommittee<mock_mainnet::Test>>::set(sync_committee_prepared);
			ValidatorsRoot::<mock_mainnet::Test>::set(get_validators_root::<SYNC_COMMITTEE_SIZE>());
			LatestFinalizedHeader::<mock_mainnet::Test>::set(FinalizedHeaderState {
				beacon_block_root: finalized_block_root,
				beacon_slot: finalized_slot,
			});
			FinalizedBeaconState::<mock_mainnet::Test>::insert(
				finalized_block_root,
				CompactBeaconState {
					slot: finalized_update.finalized_header.slot,
					block_roots_root: finalized_update.block_roots_root,
				},
			);

			assert_ok!(mock_mainnet::EthereumBeaconClient::submit_execution_header(
				mock_mainnet::RuntimeOrigin::signed(1),
				update.clone()
			));

			assert!(<ExecutionHeaders<mock_mainnet::Test>>::contains_key(
				update.execution_header.block_hash
			));
		});
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

		let participant_pubkeys = mock_mainnet::EthereumBeaconClient::find_pubkeys(
			&participant_bits,
			&milagro_pubkeys,
			true,
		);

		let signing_root = mock_mainnet::EthereumBeaconClient::signing_root(
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
}
