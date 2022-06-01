use crate::mock::{new_tester, AccountId, Ether, EtherApp, Event, Origin, System, Test};
use frame_support::{
	assert_noop, assert_ok,
	traits::fungible::{Inspect, Mutate},
};
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

use snowbridge_core::{assets::RemoteParachain, ChannelId};

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

		assert_ok!(EtherApp::mint(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			sender,
			recipient.clone(),
			amount,
			None,
		));
		assert_eq!(Ether::balance(&recipient), amount);

		assert_eq!(
			Event::EtherApp(crate::Event::<Test>::Minted(sender, recipient, amount)),
			last_event()
		);
	});
}

#[test]
fn mints_after_xcm_error() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let sender = H160::repeat_byte(7);
		let recipient: AccountId = Keyring::Bob.into();
		let amount = 10;

		assert_ok!(EtherApp::mint(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			sender,
			recipient.clone(),
			amount,
			Some(RemoteParachain { para_id: 2001, fee: 1000000u128 }),
		));
		assert_eq!(Ether::balance(&recipient), amount);

		assert_eq!(
			Event::EtherApp(crate::Event::<Test>::Minted(sender, recipient, amount)),
			last_event()
		);
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let recipient = H160::repeat_byte(2);
		let bob: AccountId = Keyring::Bob.into();

		Ether::mint_into(&bob, 500).unwrap();

		assert_ok!(EtherApp::burn(
			Origin::signed(bob.clone()),
			ChannelId::Incentivized,
			recipient.clone(),
			20
		));

		assert_eq!(Event::EtherApp(crate::Event::<Test>::Burned(bob, recipient, 20)), last_event());
	});
}

#[test]
fn should_not_burn_on_commitment_failure() {
	new_tester().execute_with(|| {
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(9);

		Ether::mint_into(&sender, 500).unwrap();

		// fill up message queue
		for _ in 0..3 {
			let _ = EtherApp::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				recipient.clone(),
				20,
			);
		}

		assert_noop!(
			EtherApp::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				recipient.clone(),
				20
			),
			snowbridge_incentivized_channel::outbound::Error::<Test>::QueueSizeLimitReached
		);
	});
}
