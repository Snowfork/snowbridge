use crate::mock::{new_tester, AccountId, Origin, Event, System, Asset, EthApp};
use frame_support::{assert_ok, assert_noop, dispatch::DispatchError};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use crate::RawEvent;

use artemis_core::{SingleAsset, ChannelId};

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let sender = H160::repeat_byte(7);
		let recipient: AccountId = Keyring::Bob.into();
		let amount = 10;
		assert_ok!(
			EthApp::mint(
				artemis_dispatch::Origin(peer_contract).into(),
				sender,
				recipient.clone(),
				amount.into()
			)
		);
		assert_eq!(Asset::balance(&recipient), amount.into());

		assert_eq!(
			Event::EthApp(RawEvent::Minted(sender, recipient, amount.into())),
			last_event()
		);
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let recipient = H160::repeat_byte(2);
		let bob: AccountId = Keyring::Bob.into();
		Asset::deposit(&bob, 500.into()).unwrap();

		assert_ok!(EthApp::burn(
			Origin::signed(bob.clone()),
			ChannelId::Incentivized,
			recipient.clone(),
			20.into()));

		assert_eq!(
			Event::EthApp(RawEvent::Burned(bob, recipient, 20.into())),
			last_event()
		);
	});
}

#[test]
fn should_not_burn_on_commitment_failure() {
	new_tester().execute_with(|| {
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(9);

		Asset::deposit(&sender, 500.into()).unwrap();

		assert_noop!(
			EthApp::burn(
				Origin::signed(sender.clone()),
				ChannelId::Basic,
				recipient.clone(),
				20.into()
			),
			DispatchError::Other("some error!")
		);
	});
}
