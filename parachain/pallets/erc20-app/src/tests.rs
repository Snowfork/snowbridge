use crate::mock::{new_tester, AccountId, Assets, Erc20App, Event, Origin, System, Test};
use frame_support::{assert_noop, assert_ok};
use snowbridge_core::{AssetId, ChannelId, MultiAsset};
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

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
		assert_ok!(Erc20App::mint(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			token,
			sender,
			recipient.clone(),
			amount.into(),
			None
		));
		assert_eq!(Assets::balance(AssetId::Token(token), &recipient), amount.into());

		assert_eq!(
			Event::Erc20App(crate::Event::<Test>::Minted(token, sender, recipient, amount.into())),
			last_event()
		);
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let token_id = H160::repeat_byte(1);
		let recipient = H160::repeat_byte(2);
		let bob: AccountId = Keyring::Bob.into();
		Assets::deposit(AssetId::Token(token_id), &bob, 500.into()).unwrap();

		assert_ok!(Erc20App::burn(
			Origin::signed(bob.clone()),
			ChannelId::Incentivized,
			token_id,
			recipient.clone(),
			20.into()
		));

		assert_eq!(
			Event::Erc20App(crate::Event::<Test>::Burned(token_id, bob, recipient, 20.into())),
			last_event()
		);
	});
}

#[test]
fn should_not_burn_on_commitment_failure() {
	new_tester().execute_with(|| {
		let token_id = H160::repeat_byte(1);
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(9);

		Assets::deposit(AssetId::Token(token_id), &sender, 500.into()).unwrap();

		for _ in 0..3 {
			let _ = Erc20App::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				token_id,
				recipient.clone(),
				20.into(),
			);
		}

		assert_noop!(
			Erc20App::burn(
				Origin::signed(sender.clone()),
				ChannelId::Incentivized,
				token_id,
				recipient.clone(),
				20.into()
			),
			snowbridge_incentivized_channel::outbound::Error::<Test>::QueueSizeLimitReached
		);
	});
}
