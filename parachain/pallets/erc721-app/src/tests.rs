use crate::mock::{self, new_tester, System, AccountId, Origin, ERC721App, NftApp};
use frame_support::{assert_ok, assert_noop, dispatch::DispatchError};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;

use artemis_ethereum::U256;
use artemis_core::nft::Nft;

use crate::Event;

fn last_event() -> mock::Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let token = H160::repeat_byte(2);
		let token_id = U256::from(1);
		let sender = H160::repeat_byte(3);
		let recipient: AccountId = Keyring::Bob.into();

		assert_ok!(ERC721App::mint(
			artemis_dispatch::Origin(peer_contract).into(),
			sender,
			recipient.clone(),
			token,
			token_id,
			"http uri".into(),
		));
		assert!(NftApp::is_owner(&recipient, 0));

		assert_eq!(
			mock::Event::erc721_app(Event::Minted(token, sender, recipient)),
			last_event()
		);
	});
}

// #[test]
// fn burn_should_emit_bridge_event() {
// 	new_tester().execute_with(|| {
// 		let token_id = H160::repeat_byte(1);
// 		let recipient = H160::repeat_byte(2);
// 		let bob: AccountId = Keyring::Bob.into();
// 		Assets::deposit(AssetId::Token(token_id), &bob, 500.into()).unwrap();

// 		assert_ok!(ERC20App::burn(
// 			Origin::signed(bob.clone()),
// 			ChannelId::Incentivized,
// 			token_id,
// 			recipient.clone(),
// 			20.into()));

// 		assert_eq!(
// 			Event::erc20_app(RawEvent::Burned(token_id, bob, recipient, 20.into())),
// 			last_event()
// 		);
// 	});
// }

// #[test]
// fn should_not_burn_on_commitment_failure() {
// 	new_tester().execute_with(|| {
// 		let token_id = H160::repeat_byte(1);
// 		let sender: AccountId = Keyring::Bob.into();
// 		let recipient = H160::repeat_byte(9);

// 		Assets::deposit(AssetId::Token(token_id), &sender, 500.into()).unwrap();

// 		assert_noop!(
// 			ERC20App::burn(
// 				Origin::signed(sender.clone()),
// 				ChannelId::Basic,
// 				token_id,
// 				recipient.clone(),
// 				20.into()
// 			),
// 			DispatchError::Other("some error!")
// 		);
// 	});
// }
