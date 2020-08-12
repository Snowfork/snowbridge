

use crate::mock::{new_tester, AccountId, GenericAsset, System, MockRuntime, TestEvent};

use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use frame_support::storage::StorageDoubleMap;

use crate::{Account, TotalIssuance};

use super::*;

fn last_event() -> TestEvent {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mint_should_increase_balance_and_total_issuance() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		assert_ok!(GenericAsset::do_mint(token_addr, &alice, 500.into()));
		assert_eq!(Account::<MockRuntime>::get(&token_addr, &alice).free, 500.into());
		assert_eq!(TotalIssuance::get(&token_addr), 500.into());

		assert_ok!(GenericAsset::do_mint(token_addr, &alice, 20.into()));
		assert_eq!(Account::<MockRuntime>::get(&token_addr, &alice).free, 520.into());
		assert_eq!(TotalIssuance::get(&token_addr), 520.into());
	});
}

#[test]
fn mint_should_raise_event() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		GenericAsset::do_mint(token_addr, &alice, 500.into()).unwrap();
		assert_eq!(TestEvent::generic_asset(RawEvent::Minted(token_addr, alice, 500.into())), last_event());
	});
}

#[test]
fn burn_should_decrease_balance_and_total_issuance() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		GenericAsset::do_mint(token_addr, &alice, 500.into()).unwrap();

		assert_ok!(GenericAsset::do_burn(token_addr, &alice, 20.into()));
		assert_eq!(Account::<MockRuntime>::get(&token_addr, &alice).free, 480.into());
		assert_eq!(TotalIssuance::get(&token_addr), 480.into());
	});
}

#[test]
fn burn_should_raise_event() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		GenericAsset::do_mint(token_addr, &alice, 500.into()).unwrap();

		GenericAsset::do_burn(token_addr, &alice, 20.into()).unwrap();
		assert_eq!(TestEvent::generic_asset(RawEvent::Burned(token_addr, alice, 20.into())), last_event());
	});
}