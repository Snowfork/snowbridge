use crate::mock::*;
use crate::Error;
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

	//let test_hex = hex!("fd5e397a84884641f53c496804f24b5276cbb8c5c9cfc2342246be8e3ce5ad02").into();

	new_tester().execute_with(|| {
		assert_err!(Ethereum2LightClient::import_header(
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
		assert_eq!(Ethereum2LightClient::is_valid_merkle_branch(
			hex!("5d0cb03baca66860a9d039c7579503c2b0e7e9d5d8d767b007e0064ce22df0c8").into(),
			vec![
				hex!("3670c5c45d82686c844a30e23854f2e32cdcadf654c285998d3267d99d7d165e").into(),
				hex!("83a0ee3b3352e98d0918d59a427670261af75b7903fdfefc73a85ad39abf8b32").into(),
				hex!("731eaeba1ccf1d442b915a537f89ae9f211677535e142b5d14750e692c7a42ca").into(),
				hex!("ac5925beb9ef24aa3c84522d168ac722a83970ca92908dbdfc9d770fca5cb659").into(),
				hex!("fee14011611c7e0c3e70c264522ee739a25c82af1530533f5eeefa18787e3dad").into(),
				hex!("5d0cb03baca66860a9d039c7579503c2b0e7e9d5d8d767b007e0064ce22df0c8").into(),
			],
			3,
			7,
			hex!("5d0cb03baca66860a9d039c7579503c2b0e7e9d5d8d767b007e0064ce22df0c8").into()
		), false);
	});
}