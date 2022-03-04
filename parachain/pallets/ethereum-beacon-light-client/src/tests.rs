use crate as ethereum_beacon_light_client;
use crate::mock::*;
use crate::Error;
use frame_support::{assert_err, assert_ok};
use hex_literal::hex;

#[test]
fn it_gets_an_update() {
	let update = get_update();

	new_tester().execute_with(|| {
		assert_err!(
			EthereumBeaconLightClient::light_client_update(
				Origin::signed(1),
				update,
				897,
				hex!("043db0d9a83813551ee2f33450d23797757d430911a9320530ad8a0eabc43efb").into()
			),
			Error::<Test>::AncientHeader
		);
	});
}

#[test]
pub fn test_is_valid_merkle_proof() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::is_valid_merkle_branch(
				hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
				vec![
					hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
					hex!("5f6f02af29218292d21a69b64a794a7c0873b3e0f54611972863706e8cbdf371").into(),
					hex!("e7125ff9ab5a840c44bedb4731f440a405b44e15f2d1a89e27341b432fabe13d").into(),
					hex!("002c1fe5bc0bd62db6f299a582f2a80a6d5748ccc82e7ed843eaf0ae0739f74a").into(),
					hex!("d2dc4ba9fd4edff6716984136831e70a6b2e74fca27b8097a820cbbaa5a6e3c3").into(),
					hex!("91f77a19d8afa4a08e81164bb2e570ecd10477b3b65c305566a6d2be88510584").into(),
				],
				6,
				41,
				hex!("e46559327592741956f6beaa0f52e49625eb85dce037a0bd2eff333c743b287f").into()
			),
			true
		);
	});
}

#[test]
pub fn test_is_not_valid_merkle_proof() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::is_valid_merkle_branch(
				hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
				vec![
					hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
					hex!("5f6f02af29218292d21a69b64a794a7c0873b3e0f54611972863706e8cbdf371").into(),
					hex!("e7125ff9ab5a840c44bedb4731f440a405b44e15f2d1a89e27341b432fabe13d").into(),
					hex!("333c1fe5bc0bd62db6f299a582f2a80a6d5748ccc82e7ed843eaf0ae0739f74a").into(),
					hex!("d2dc4ba9fd4edff6716984136831e70a6b2e74fca27b8097a820cbbaa5a6e3c3").into(),
					hex!("91f77a19d8afa4a08e81164bb2e570ecd10477b3b65c305566a6d2be88510584").into(),
				],
				6,
				41,
				hex!("e46559327592741956f6beaa0f52e49625eb85dce037a0bd2eff333c743b287f").into()
			),
			false
		);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify() {
	new_tester().execute_with(|| {
		assert_ok!(EthereumBeaconLightClient::bls_fast_aggregate_verify(
			vec![
				hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into(),
				hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into(),
				hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into(),
				hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into(),
			],
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		));
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_point() {
	new_tester().execute_with(|| {
		assert_err!(EthereumBeaconLightClient::bls_fast_aggregate_verify(
			vec![
				hex!("973eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into(),
				hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into(),
				hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into(),
				hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into(),
			],
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		), Error::<Test>::InvalidSignaturePoint);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_message() {
	new_tester().execute_with(|| {
		assert_err!(EthereumBeaconLightClient::bls_fast_aggregate_verify(
			vec![
				hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into(),
				hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into(),
				hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into(),
				hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into(),
			],
			hex!("99241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		), Error::<Test>::SignatureVerificationFailed);
	});
}

#[test]
pub fn test_bls_fast_aggregate_verify_invalid_signature() {
	new_tester().execute_with(|| {
		assert_err!(EthereumBeaconLightClient::bls_fast_aggregate_verify(
			vec![
				hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into(),
				hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into(),
				hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into(),
				hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into(),
			],
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("c204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		), Error::<Test>::InvalidSignature);
	});
}

#[test]
pub fn test_hash_tree_root() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::hash_tree_root(
				ethereum_beacon_light_client::BeaconBlockHeader {
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
			),
			hex!("7d42595818709e805dd2fa710a2d2c1f62576ef1ab7273941ac9130fb94b91f7").into()
		);
	});
}

#[test]
pub fn test_hash_tree_root_with_root_value() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::hash_tree_root(hex!(
				"6807a67bb39d237056f96a6c04cbfcb244b7ffbe763e817d643bd756e7df0cf0"
			)),
			hex!("6807a67bb39d237056f96a6c04cbfcb244b7ffbe763e817d643bd756e7df0cf0").into()
		);
	});
}

#[test]
pub fn test_hash_tree_root_slot() {
	let slot: ethereum_beacon_light_client::Slot = 2;
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::hash_tree_root(slot),
			hex!("0200000000000000000000000000000000000000000000000000000000000000").into()
		);
	});
}

#[test]
pub fn test_compute_epoch_at_slot() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::compute_epoch_at_slot(0), 0);
	});
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::compute_epoch_at_slot(4), 0);
	});
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::compute_epoch_at_slot(36), 1);
	});
}

#[test]
pub fn test_get_subtree_index() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::get_subtree_index(105), 41);
	});
}

#[test]
pub fn test_floor_log2() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::floorlog2(4), 2);
	});
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::floorlog2(16), 4);
	});
}

#[test]
pub fn test_get_safety_threshold() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::get_safety_threshold(30, 55), 55);
	});
}

#[test]
pub fn test_get_sync_committee_sum() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::get_sync_committee_sum(vec![0, 1, 0, 1, 1, 0, 1, 0, 1]),
			5
		);
	});
}

#[test]
pub fn test_compute_domain() {
	new_tester().execute_with(|| {
		assert_eq!(
			EthereumBeaconLightClient::compute_domain(
				hex!("05000000").into(),
				hex!("00000001").into(),
				hex!("5dec7ae03261fde20d5b024dfabce8bac3276c9a4908e23d50ba8c9b50b0adff").into(),
			),
			hex!("0500000046324489ceb6ada6d118eacdbe94f49b1fcb49d5481a685979670c7c").into()
		);
	});
}
