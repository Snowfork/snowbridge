use crate::mock::{new_tester, AccountId, Origin, TokenDealer};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;

use crate::{XCurrencyId, CurrencyId, ChainId};

use xcm::v0::NetworkId;

#[test]
fn it_executes_message() {

	let x_currency_id = XCurrencyId {
		chain_id: ChainId::ParaChain(100.into()),
		currency_id: CurrencyId::ETH
	};

	let network_id = NetworkId::Any;

	new_tester().execute_with(|| {
		let alice: AccountId = Keyring::Bob.into();
		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(
			TokenDealer::transfer_to_parachain(
				Origin::signed(alice.clone()),
				x_currency_id,
				200.into(),
				bob,
				network_id,
				100
			)
		);
	})
}
