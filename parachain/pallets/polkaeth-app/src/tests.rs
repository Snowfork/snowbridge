use crate::{mock::*};
use frame_support::{assert_ok};

#[test]
fn it_mints() {
	new_tester().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(PolkaETHModule::mint(origin, BOB, 42));
		assert_eq!(BalancesPolkaETH::free_balance(BOB), 1042);
	});
}

#[test]
fn it_burns() {
	new_tester().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(PolkaETHModule::burn(origin, 500, true));
		assert_eq!(BalancesPolkaETH::free_balance(ALICE), 500);
	});
}
