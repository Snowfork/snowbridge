use crate::mock::{new_tester, MockEvent, MockRuntime, System, AccountId, Origin, Assets, ERC20};
use frame_support::{assert_ok};
use frame_system as system;
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use hex_literal::hex;
use codec::Decode;

use artemis_core::{AssetId, MultiAsset};

use crate::RawEvent;

use crate::payload::InPayload;

type TestAccountId = <MockRuntime as system::Trait>::AccountId;

const RECIPIENT_ADDR_BYTES: [u8; 32] = hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"];

fn last_event() -> MockEvent {
	System::events().pop().expect("Event expected").event
}

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let token_addr = H160::repeat_byte(1);

		let recipient_addr = TestAccountId::decode(&mut &RECIPIENT_ADDR_BYTES[..]).unwrap();
		let event: InPayload<TestAccountId> = InPayload {
			sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
			recipient_addr,
			token_addr,
			amount: 10.into(),
		};

		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(ERC20::handle_event(event));
		assert_eq!(Assets::balance(AssetId::Token(token_addr), &bob), 10.into());
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let token_id = H160::repeat_byte(1);
		let recipient = H160::repeat_byte(2);
		let bob: AccountId = Keyring::Bob.into();
		Assets::deposit(AssetId::Token(token_id), &bob, 500.into()).unwrap();

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
