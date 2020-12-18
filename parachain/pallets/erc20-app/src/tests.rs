<<<<<<< HEAD
use crate::mock::{new_tester, MockEvent, MockRuntime, System, AccountId, Origin, Assets, ERC20};
use frame_support::{assert_ok};
=======
use artemis_core::{Application, VerificationOutput};
use crate::mock::{new_tester, MockEvent, MockRuntime, System, AccountId, Origin, Asset, ERC20};
use frame_support::{assert_err, assert_ok};
>>>>>>> Move verification check to Application::handle
use frame_system as system;
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use hex_literal::hex;
use codec::Decode;

use artemis_core::{AssetId, MultiAsset};

use crate::{Error, RawEvent};

use crate::payload::InPayload;

type TestAccountId = <MockRuntime as system::Trait>::AccountId;

const RECIPIENT_ADDR_BYTES: [u8; 32] = hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"];

const PAYLOAD: [u8; 187] = hex!("
	f8b994c3a1ca063da8d4d3b2c697316ea6e69ccd263a44e1a0be9215fdb4
	23dfc80cce917dc48fa52d3e247875e3d7cea229d3f28661ad0f60b88000
	0000000000000000000000cffeaaf7681c89285d65cfbe808b80e5026965
	73d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5
	6da27d000000000000000000000000f465670390f5214ed43d5027f31ed3
	3764f0448700000000000000000000000000000000000000000000000000
	00000000000002
");


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

#[test]
fn handle_should_reject_invalid_verification() {
	new_tester().execute_with(|| {
		assert_err!(
			ERC20::handle(&PAYLOAD, &VerificationOutput::None),
			Error::<MockRuntime>::InvalidVerification,
		);
		assert_err!(
			ERC20::handle(&PAYLOAD, &VerificationOutput::Receipt(Default::default())),
			Error::<MockRuntime>::InvalidVerification,
		);
	})
}
