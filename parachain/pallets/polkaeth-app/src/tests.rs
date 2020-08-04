use crate::{mock::*};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;

#[test]
fn it_mints() {
	new_tester().execute_with(|| {
		let origin = Origin::signed(Keyring::Alice.into());
		assert_ok!(PolkaETHModule::mint(origin, Keyring::Bob.into(), 42));

		let bob: AccountId = Keyring::Bob.into();

		assert_eq!(BalancesPolkaETH::free_balance(bob), 1042);
	});
}

#[test]
fn it_burns() {
	new_tester().execute_with(|| {
		let origin = Origin::signed(Keyring::Alice.into());
		assert_ok!(PolkaETHModule::burn(origin, 500, true));

		let alice: AccountId = Keyring::Alice.into();

		assert_eq!(BalancesPolkaETH::free_balance(alice), 500);
	});
}

#[test]
fn it_handles_message() {
	new_tester().execute_with(|| {
		let origin = Origin::signed(Keyring::Alice.into());
		assert_ok!(PolkaETHModule::burn(origin, 500, true));

		let alice: AccountId = Keyring::Alice.into();
	
		assert_eq!(BalancesPolkaETH::free_balance(alice), 500);
	});
}
