use crate::mock::{new_tester, Event, System, AccountId, Origin, Assets, Erc20App};
use frame_support::{assert_ok, assert_noop, dispatch::DispatchError};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use artemis_core::{ChannelId, AssetId, MultiAsset};

use crate::RawEvent;

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
		assert_ok!(
			Erc20App::mint(
				artemis_dispatch::Origin(peer_contract).into(),
				token,
				sender,
				recipient.clone(),
				amount.into()
			)
		);
		assert_eq!(Assets::balance(AssetId::Token(token), &recipient), amount.into());

		assert_eq!(
			Event::Erc20App(RawEvent::Minted(token, sender, recipient, amount.into())),
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
			20.into()));

		assert_eq!(
			Event::Erc20App(RawEvent::Burned(token_id, bob, recipient, 20.into())),
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

		assert_noop!(
			Erc20App::burn(
				Origin::signed(sender.clone()),
				ChannelId::Basic,
				token_id,
				recipient.clone(),
				20.into()
			),
			DispatchError::Other("some error!")
		);
	});
}
