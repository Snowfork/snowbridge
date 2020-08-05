use crate::{mock::*};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;

use frame_support::storage::StorageDoubleMap;

use crate::FreeBalance;

#[test]
fn it_mints() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		assert_ok!(PolkaERC20::do_mint(token_addr, &alice, 500));

		assert_eq!(FreeBalance::<MockRuntime>::get(&token_addr, &alice), 500);

	});
}
