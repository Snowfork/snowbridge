use crate::mock::{self, new_tester, AccountId, Erc721App, NftApp, Origin, System};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

use snowbridge_core::{
	nft::{ERC721TokenData, Nft},
	ChannelId,
};
use snowbridge_ethereum::U256;

use crate::Event;

fn last_event() -> mock::Event {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let token_contract = H160::repeat_byte(2);
		let token_id = U256::from(1);
		let sender = H160::repeat_byte(3);
		let recipient: AccountId = Keyring::Bob.into();

		assert_ok!(Erc721App::mint(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			sender,
			recipient.clone(),
			token_contract,
			token_id,
			"http uri".into(),
		));
		assert!(NftApp::is_owner(&recipient, 0));

		assert_eq!(
			mock::Event::Erc721App(Event::Minted(token_contract, token_id, sender, recipient)),
			last_event()
		);
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let token = 0u64;
		let recipient = H160::repeat_byte(2);
		let token_contract = H160::repeat_byte(4);
		let token_id = U256::one();
		let bob: AccountId = Keyring::Bob.into();
		let data = ERC721TokenData { token_contract, token_id };

		NftApp::mint(&bob, Vec::<u8>::new(), data).unwrap();

		assert_ok!(Erc721App::burn(
			Origin::signed(bob.clone()),
			ChannelId::Incentivized,
			token,
			recipient.clone()
		));

		assert_eq!(
			mock::Event::Erc721App(Event::Burned(token_contract, token_id, bob, recipient)),
			last_event()
		);
	});
}

#[test]
fn should_not_burn_on_commitment_failure() {
	new_tester().execute_with(|| {
		let token_id = 0u64;
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(9);
		let token_contract = H160::repeat_byte(3);
		let data = ERC721TokenData { token_contract, token_id: U256::one() };

		NftApp::mint(&sender, vec![0, 1, 2], data).unwrap();

		assert_noop!(
			Erc721App::burn(
				Origin::signed(sender.clone()),
				ChannelId::Basic,
				token_id,
				recipient.clone()
			),
			DispatchError::Other("some error!")
		);
	});
}
