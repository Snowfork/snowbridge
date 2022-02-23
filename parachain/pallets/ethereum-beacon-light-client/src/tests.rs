use crate::mock::*;
use crate::Error;
use crate as ethereum_beacon_light_client;
use frame_support::{assert_err, assert_ok};
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
		assert_err!(EthereumBeaconLightClient::import_header(
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