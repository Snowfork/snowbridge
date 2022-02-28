use crate::mock::*;
use crate::Error;
use crate as ethereum_beacon_light_client;
use frame_support::{assert_err, assert_ok};
use hex_literal::hex;

use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};

#[test]
fn it_gets_an_update() {
	let update = get_update();

	new_tester().execute_with(|| {
		assert_err!(EthereumBeaconLightClient::light_client_update(
			Origin::signed(1),
			update,
			897,
			hex!("043db0d9a83813551ee2f33450d23797757d430911a9320530ad8a0eabc43efb").into()
		), Error::<Test>::AncientHeader);
	});
}

#[test]
pub fn test_is_valid_merkle_proof() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::is_valid_merkle_branch(
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
		), true);
	});
}

#[test]
pub fn test_is_not_valid_merkle_proof() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::is_valid_merkle_branch(
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
		), false);
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
		assert_eq!(EthereumBeaconLightClient::hash_tree_root(
			ethereum_beacon_light_client::BeaconBlockHeader{
				slot: 3,
				proposer_index: 2,
				parent_root: hex!("796ea53efb534eab7777809cc5ee2d84e7f25024b9d0c4d7e5bcaab657e4bdbd").into(),
				state_root: hex!("ba3ff080912be5c9c158b2e962c1b39a91bc0615762ba6fa2ecacafa94e9ae0a").into(),
				body_root: hex!("a18d7fcefbb74a177c959160e0ee89c23546482154e6831237710414465dcae5").into(),
			}
		), hex!("7d42595818709e805dd2fa710a2d2c1f62576ef1ab7273941ac9130fb94b91f7").into());
	});
}

#[test]
pub fn test_hash_tree_root_checkpoint() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::hash_tree_root(
			ethereum_beacon_light_client::Checkpoint{
				epoch: 1,
				root: hex!("ffeffd88ce0305f2e8518c3ac9368e9ec493460cad83f13c54566e3ee0938b83").into(),
			}
		), hex!("96b6f4404b29574e19efc3bddc0967fa76dde2217780b0f09c58f58924ec0540").into());
	});
}

#[test]
pub fn test_ssz_encode() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::ssz_encode(
			ethereum_beacon_light_client::Checkpoint{
				epoch: 1,
				root: hex!("ffeffd88ce0305f2e8518c3ac9368e9ec493460cad83f13c54566e3ee0938b83").into(),
			}
		), hex!("0100000000000000ffeffd88ce0305f2e8518c3ac9368e9ec493460cad83f13c54566e3ee0938b83").to_vec());
	});
}

#[test]
pub fn test_ssz_beacon_header() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::ssz_encode(
			ethereum_beacon_light_client::BeaconBlockHeader{
				slot: 3,
				proposer_index: 6,
				parent_root: hex!("94ea256cd0407e5b505bc5206bcd485e7367fe52bb0541fda3970b1a3cf651a2").into(),
				state_root: hex!("478e8580b66fe6dddad826acba5dce35051a7b671b4409207816c45891388a67").into(),
				body_root: hex!("aed0abba2c0d37380252e876fab4468b96d9227204845c398f330e618df7de76").into(),
			}
		), hex!("0300000000000000060000000000000094ea256cd0407e5b505bc5206bcd485e7367fe52bb0541fda3970b1a3cf651a2478e8580b66fe6dddad826acba5dce35051a7b671b4409207816c45891388a67aed0abba2c0d37380252e876fab4468b96d9227204845c398f330e618df7de76").to_vec());
	});
}

#[test]
pub fn test_hash_tree_root_with_root_value() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::hash_tree_root(
			hex!("6807a67bb39d237056f96a6c04cbfcb244b7ffbe763e817d643bd756e7df0cf0")
		), hex!("6807a67bb39d237056f96a6c04cbfcb244b7ffbe763e817d643bd756e7df0cf0").into());
	});
}

#[test]
pub fn test_hash_tree_root_slot() {
	let slot: ethereum_beacon_light_client::Slot = 2;
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::hash_tree_root(
			slot
		), hex!("0200000000000000000000000000000000000000000000000000000000000000").into());
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