use crate::mock::{new_tester, MockEvent, System, AccountId, Origin, GenericAsset, ERC20};
use crate::{APP_ID, TransferEvent};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use hex::FromHex;

use codec::Encode;

use artemis_ethereum::Event;

use pallet_bridge as bridge;

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
		let token_addr = H160::repeat_byte(1);

		let event = Event::SendERC20 {
			sender: "cffeaaf7681c89285d65cfbe808b80e502696573".parse().unwrap(),
			recipient: to_account_id("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
			token: token_addr,
			amount: 10.into(),
			nonce: 1
		};

		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(ERC20::handle_event(event));
		assert_eq!(GenericAsset::free_balance(token_addr, &bob), 10.into());
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let token_addr = H160::repeat_byte(1);
		let bob: AccountId = Keyring::Bob.into();
		GenericAsset::do_mint(token_addr, &bob, 500.into()).unwrap();

		assert_ok!(ERC20::burn(
			Origin::signed(bob.clone()),
			token_addr,
			20.into()));

		let app_event: TransferEvent<AccountId> = TransferEvent::Burned(token_addr, bob.clone(), 20.into());

		assert_eq!(MockEvent::bridge(bridge::RawEvent::Transfer(*APP_ID, app_event.encode())), last_event());

	});
}
