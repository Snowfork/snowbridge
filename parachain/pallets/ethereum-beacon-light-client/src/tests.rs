use crate::mock::*;
use crate::Error;
use crate as ethereum_beacon_light_client;
use frame_support::assert_err;
use hex_literal::hex;

use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};

#[test]
fn it_works() {
	new_tester().execute_with(|| {
		assert_eq!(true, true);
	});
}

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

#[derive(PartialEq, Debug, Encode, Decode)]
struct Foo {
    a: u64,
    b: Vec<u16>,
}
#[test]
pub fn test_ssz() {
	let foo = Foo {
        a: 42,
        b: vec![1, 3, 3, 7]
    };

    let ssz_bytes: Vec<u8> = foo.as_ssz_bytes();

    let decoded_foo = Foo::from_ssz_bytes(&ssz_bytes).unwrap();

    assert_eq!(foo, decoded_foo);
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
/// FIX not passing yet
pub fn test_bls_fast_aggregate_verify() {
	new_tester().execute_with(|| {
		assert_eq!(EthereumBeaconLightClient::bls_fast_aggregate_verify(
			vec![
				hex!("a73eb991aa22cdb794da6fcde55a427f0a4df5a4a70de23a988b5e5fc8c4d844f66d990273267a54dd21579b7ba6a086").into(),
				hex!("b29043a7273d0a2dbc2b747dcf6a5eccbd7ccb44b2d72e985537b117929bc3fd3a99001481327788ad040b4077c47c0d").into(),
				hex!("b928f3beb93519eecf0145da903b40a4c97dca00b21f12ac0df3be9116ef2ef27b2ae6bcd4c5bc2d54ef5a70627efcb7").into(),
				hex!("9446407bcd8e5efe9f2ac0efbfa9e07d136e68b03c5ebc5bde43db3b94773de8605c30419eb2596513707e4e7448bb50").into(),
			],
			hex!("69241e7146cdcc5a5ddc9a60bab8f378c0271e548065a38bcc60624e1dbed97f").into(),
			hex!("b204e9656cbeb79a9a8e397920fd8e60c5f5d9443f58d42186f773c6ade2bd263e2fe6dbdc47f148f871ed9a00b8ac8b17a40d65c8d02120c00dca77495888366b4ccc10f1c6daa02db6a7516555ca0665bca92a647b5f3a514fa083fdc53b6e").to_vec(),
		), false);
	});
}