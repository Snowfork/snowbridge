use crate::mock::*;
use crate::Error;
use frame_support::assert_err;
use hex_literal::hex;

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
			hex!("fd5e397a84884641f53c496804f24b5276cbb8c5c9cfc2342246be8e3ce5ad02").into()
		), Error::<Test>::AncientHeader);
	});
}