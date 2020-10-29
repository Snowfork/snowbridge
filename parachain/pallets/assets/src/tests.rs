

use crate::mock::{new_tester, AccountId, Asset, Origin, System, MockRuntime, TestEvent};

use frame_support::{assert_ok, assert_noop};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use frame_support::storage::StorageDoubleMap;

use crate::{Account, TotalIssuance};

use super::*;

fn last_event() -> TestEvent {
	System::events().pop().expect("Event expected").event
}

fn set_balance<T>(asset_id: &H160, account_id: &AccountId, amount: T)
	where T : Into<U256> + Copy
{
	let free = amount.into();
	let account = AccountData { free };
	Account::<MockRuntime>::insert(&asset_id, &account_id, &account);
	TotalIssuance::insert(&asset_id, free);
}

#[test]
fn mint_should_increase_balance_and_total_issuance() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		assert_ok!(Asset::do_mint(asset_id, &alice, 500.into()));
		assert_eq!(Account::<MockRuntime>::get(&asset_id, &alice).free, 500.into());
		assert_eq!(TotalIssuance::get(&asset_id), 500.into());

		assert_ok!(Asset::do_mint(asset_id, &alice, 20.into()));
		assert_eq!(Account::<MockRuntime>::get(&asset_id, &alice).free, 520.into());
		assert_eq!(TotalIssuance::get(&asset_id), 520.into());
	});
}

#[test]
fn mint_should_raise_event() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		assert_ok!(Asset::do_mint(asset_id, &alice, 500.into()));
		assert_eq!(TestEvent::generic_asset(RawEvent::Minted(asset_id, alice, 500.into())), last_event());
	});
}

#[test]
fn mint_should_raise_total_overflow_error() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		TotalIssuance::insert(&asset_id, U256::MAX);

		assert_noop!(
			Asset::do_mint(asset_id, &alice, U256::one()),
			Error::<MockRuntime>::TotalMintingOverflow
		);

	});
}

#[test]
fn mint_should_raise_free_overflow_error() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		let account = AccountData { free: U256::MAX };
		Account::<MockRuntime>::insert(&asset_id, &alice, &account);

		assert_noop!(
			Asset::do_mint(asset_id, &alice, U256::one()),
			Error::<MockRuntime>::FreeMintingOverflow
		);

	});
}

#[test]
fn burn_should_decrease_balance_and_total_issuance() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		set_balance(&asset_id, &alice, 500);

		assert_ok!(Asset::do_burn(asset_id, &alice, 20.into()));
		assert_eq!(Account::<MockRuntime>::get(&asset_id, &alice).free, 480.into());
		assert_eq!(TotalIssuance::get(&asset_id), 480.into());
	});
}

#[test]
fn burn_should_raise_event() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		set_balance(&asset_id, &alice, 500);

		assert_ok!(Asset::do_burn(asset_id, &alice, 20.into()));
		assert_eq!(TestEvent::generic_asset(RawEvent::Burned(asset_id, alice, 20.into())), last_event());
	});
}

#[test]
fn burn_should_raise_total_underflow_error() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		TotalIssuance::insert(&asset_id, U256::one());

		assert_noop!(
			Asset::do_burn(asset_id, &alice, 10.into()),
			Error::<MockRuntime>::TotalBurningUnderflow
		);

	});
}

#[test]
fn burn_should_raise_free_underflow_error() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		let account = AccountData { free: U256::one() };
		Account::<MockRuntime>::insert(&asset_id, &alice, &account);

		assert_noop!(
			Asset::do_burn(asset_id, &alice, 10.into()),
			Error::<MockRuntime>::TotalBurningUnderflow
		);

	});
}

#[test]
fn transfer_free_balance() {
	new_tester().execute_with(|| {

		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(Asset::do_mint(asset_id, &alice, 500.into()));
		assert_ok!(Asset::do_mint(asset_id, &bob, 500.into()));
		assert_ok!(Asset::transfer(Origin::signed(alice.clone()), asset_id, bob.clone(), 250.into()));

		assert_eq!(Account::<MockRuntime>::get(&asset_id, &alice).free, 250.into());
		assert_eq!(Account::<MockRuntime>::get(&asset_id, &bob).free, 750.into());
		assert_eq!(TotalIssuance::get(&asset_id), 1000.into());
	});
}

#[test]
fn transfer_should_raise_insufficient_balance() {
	new_tester().execute_with(|| {

		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(Asset::do_mint(asset_id, &alice, 500.into()));
		assert_ok!(Asset::do_mint(asset_id, &bob, 500.into()));

		assert_noop!(
			Asset::transfer(Origin::signed(alice.clone()), asset_id, bob.clone(), 1000.into()),
			Error::<MockRuntime>::InsufficientBalance,
		);

	});
}

// In theory this case will never occur since Sum(Account.free) == TotalIssuance and thus
// overflow should be impossible
#[test]
fn transfer_should_raise_overflow() {
	new_tester().execute_with(|| {
		let asset_id = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		let bob: AccountId = Keyring::Bob.into();

		let account = AccountData { free: U256::one() };
		Account::<MockRuntime>::insert(&asset_id, &alice, &account);

		let account = AccountData { free: U256::MAX };
		Account::<MockRuntime>::insert(&asset_id, &bob, &account);

		assert_noop!(
			Asset::transfer(Origin::signed(alice.clone()), asset_id, bob.clone(), 1.into()),
			Error::<MockRuntime>::FreeTransferOverflow,
		);

	});
}
