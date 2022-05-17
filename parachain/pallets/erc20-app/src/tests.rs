use crate::mock::{new_tester, AccountId, Assets, Erc20App, Event, Origin, System, Test};
use frame_support::{assert_noop, assert_ok};
use snowbridge_core::{assets::RemoteParachain, ChannelId};
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

use frame_support::traits::tokens::fungibles::Mutate;

use crate::AssetId;

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let token = H160::repeat_byte(2);
		let sender = H160::repeat_byte(3);
		let recipient: AccountId = Keyring::Bob.into();
		let amount = 10;

		// create asset
		assert_ok!(Erc20App::create(snowbridge_dispatch::RawOrigin(peer_contract).into(), token,));

		assert_ok!(Erc20App::mint(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			token,
			sender,
			recipient.clone(),
			amount,
			None
		));
		assert_eq!(Assets::balance(<AssetId<Test>>::get(token).unwrap(), &recipient), amount);

		assert_eq!(
			Event::Erc20App(crate::Event::<Test>::Minted(token, sender, recipient, amount)),
			last_event()
		);
	});
}

#[test]
fn mints_after_xcm_failure() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let token = H160::repeat_byte(2);
		let sender = H160::repeat_byte(3);
		let recipient: AccountId = Keyring::Bob.into();
		let amount = 10;

		// create asset
		assert_ok!(Erc20App::create(snowbridge_dispatch::RawOrigin(peer_contract).into(), token,));

		assert_ok!(Erc20App::mint(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			token,
			sender,
			recipient.clone(),
			amount,
			Some(RemoteParachain { para_id: 2001, fee: 1000000u128 }),
		));
		assert_eq!(Assets::balance(<AssetId<Test>>::get(token).unwrap(), &recipient), amount);

		assert_eq!(
			Event::Erc20App(crate::Event::<Test>::Minted(token, sender, recipient, amount)),
			last_event()
		);
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let token_id = H160::repeat_byte(2);
		let recipient = H160::repeat_byte(3);
		let bob: AccountId = Keyring::Bob.into();

		// create asset
		assert_ok!(Erc20App::create(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			token_id,
		));

		Assets::mint_into(<AssetId<Test>>::get(token_id).unwrap(), &bob, 500).unwrap();

		assert_ok!(Erc20App::burn(
			Origin::signed(bob.clone()),
			ChannelId::Incentivized,
			token_id,
			recipient.clone(),
			20
		));

		assert_eq!(
			Event::Erc20App(crate::Event::<Test>::Burned(token_id, bob, recipient, 20)),
			last_event()
		);
	});
}

#[test]
fn should_not_burn_on_commitment_failure() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let token_id = H160::repeat_byte(2);
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(9);

		// create asset
		assert_ok!(Erc20App::create(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			token_id,
		));

		Assets::mint_into(<AssetId<Test>>::get(token_id).unwrap(), &sender, 500).unwrap();

		for _ in 0..3 {
			let _ = Erc20App::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				token_id,
				recipient.clone(),
				20,
			);
		}

		assert_noop!(
			Erc20App::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				token_id,
				recipient.clone(),
				20
			),
			snowbridge_incentivized_channel::outbound::Error::<Test>::QueueSizeLimitReached
		);
	});
}
