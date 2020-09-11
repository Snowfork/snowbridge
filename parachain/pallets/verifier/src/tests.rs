use crate::mock::{new_tester, AccountId, Origin, MockEvent, System, Asset, ETH};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use hex::FromHex;
use sp_core::H160;

use crate::RawEvent;

use artemis_ethereum::Event as EthereumEvent;


fn last_event() -> MockEvent {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let ferdie: AccountId = Keyring::Ferdie.into();


	});
}
