use crate::mock::{new_tester, AccountId, Asset, EthApp, Event, Origin, System, Test};
use frame_support::{assert_noop, assert_ok};
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

use snowbridge_core::{ChannelId, SingleAsset};

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
		assert_ok!(EthApp::mint(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			sender,
			recipient.clone(),
			amount.into(),
			None,
		));
		assert_eq!(Asset::balance(&recipient), amount.into());

		assert_eq!(
			Event::EthApp(crate::Event::<Test>::Minted(sender, recipient, amount.into())),
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
			20.into()
		));

		assert_eq!(
			Event::EthApp(crate::Event::<Test>::Burned(bob, recipient, 20.into())),
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

		// fill up message queue
		for _ in 0..3 {
			let _ = EthApp::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				recipient.clone(),
				20.into(),
			);
		}

		assert_noop!(
			EthApp::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				recipient.clone(),
				20.into()
			),
			snowbridge_incentivized_channel::outbound::Error::<Test>::QueueSizeLimitReached
		);
	});
}
