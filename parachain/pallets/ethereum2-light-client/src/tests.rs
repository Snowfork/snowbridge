use crate::mock::*;
use frame_support::assert_ok;

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
		assert_ok!(Ethereum2LightClient::import_header(
			Origin::signed(1),
			update,
		));
	});
}