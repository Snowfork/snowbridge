use crate::mock::{new_tester, MockEvent, System, AccountId, Origin, Asset, ERC20};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use hex_literal::hex;

use crate::RawEvent;

use crate::payload::Payload;

fn last_event() -> MockEvent {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let token_addr = H160::repeat_byte(1);

		let event = Payload {
			sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
			recipient_addr: hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"],
			token_addr: token_addr,
			amount: 10.into(),
		};

		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(ERC20::handle_event(event));
		assert_eq!(Asset::free_balance(token_addr, &bob), 10.into());
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let token_id = H160::repeat_byte(1);
		let recipient = H160::repeat_byte(2);
		let bob: AccountId = Keyring::Bob.into();
		Asset::do_mint(token_id, &bob, 500.into()).unwrap();

		assert_ok!(ERC20::burn(
			Origin::signed(bob.clone()),
			token_id,
			recipient,
			20.into()));

		assert_eq!(
			MockEvent::test_events(RawEvent::Transfer(token_id, bob, recipient, 20.into())),
			last_event()
		);

	});
}
