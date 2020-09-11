use crate::mock::{new_tester, AccountId, Origin, MockEvent, System, Asset, ETH};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use hex::FromHex;
use sp_core::H160;

use crate::RawEvent;

use crate::payload::Payload;

fn last_event() -> MockEvent {
	System::events().pop().expect("Event expected").event
}

fn to_account_id(hexaddr: &str) -> [u8; 32] {
	let mut buf: [u8; 32] = [0; 32];
	let bytes: Vec<u8> = hexaddr.from_hex().unwrap();
	buf.clone_from_slice(&bytes);
	buf
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let bob: AccountId = Keyring::Bob.into();

		let event = Payload {
			sender_addr: "cffeaaf7681c89285d65cfbe808b80e502696573".parse().unwrap(),
			recipient_addr: to_account_id("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
			amount: 10.into(),
		};

		assert_ok!(ETH::handle_event(event));
		assert_eq!(Asset::free_balance(H160::zero(), &bob), 10.into());
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();
		let recipient = H160::repeat_byte(2);
		let bob: AccountId = Keyring::Bob.into();
		Asset::do_mint(token_addr, &bob, 500.into()).unwrap();

		assert_ok!(ETH::burn(
			Origin::signed(bob.clone()),
			recipient,
			20.into()));

		assert_eq!(
			MockEvent::test_events(RawEvent::Transfer(bob, recipient, 20.into())),
			last_event()
		);

	});
}
