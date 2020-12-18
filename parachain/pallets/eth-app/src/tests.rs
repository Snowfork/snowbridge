use artemis_core::{Application, VerificationOutput};
use crate::mock::{new_tester, AccountId, Origin, MockEvent, MockRuntime, System, Asset, ETH};
use frame_support::{assert_err, assert_ok};
use frame_system as system;
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use hex_literal::hex;
use codec::Decode;
use crate::{Error, RawEvent};

use artemis_core::SingleAsset;

use crate::payload::InPayload;

fn last_event() -> MockEvent {
	System::events().pop().expect("Event expected").event
}

const RECIPIENT_ADDR_BYTES: [u8; 32] = hex!["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"];

const PAYLOAD: [u8; 155] = hex!("
	f899947c5c2fb581612f040ebf9e74f94c9eac8681a95fe1a0691df88ac0
	2f64f3b39fb1b52b940a2730e41ae20f39eec131634df2f8edce77b86000
	0000000000000000000000cffeaaf7681c89285d65cfbe808b80e5026965
	73d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5
	6da27d00000000000000000000000000000000000000000000000000038d
	7ea4c68000
");

type TestAccountId = <MockRuntime as system::Trait>::AccountId;

#[test]
fn mints_after_handling_ethereum_event() {
	new_tester().execute_with(|| {
		let bob: AccountId = Keyring::Bob.into();

		let recipient_addr = TestAccountId::decode(&mut &RECIPIENT_ADDR_BYTES[..]).unwrap();
		let event: InPayload<TestAccountId> = InPayload {
			sender_addr: hex!["cffeaaf7681c89285d65cfbe808b80e502696573"].into(),
			recipient_addr,
			amount: 10.into(),
		};

		assert_ok!(ETH::handle_event(event));
		assert_eq!(Asset::balance(&bob), 10.into());
	});
}

#[test]
fn burn_should_emit_bridge_event() {
	new_tester().execute_with(|| {
		let recipient = H160::repeat_byte(2);
		let bob: AccountId = Keyring::Bob.into();
		Asset::deposit(&bob, 500.into()).unwrap();

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


#[test]
fn handle_should_reject_invalid_verification() {
	new_tester().execute_with(|| {
		assert_err!(
			ETH::handle(&PAYLOAD, &VerificationOutput::None),
			Error::<MockRuntime>::InvalidVerification,
		);
		assert_err!(
			ETH::handle(&PAYLOAD, &VerificationOutput::Receipt(Default::default())),
			Error::<MockRuntime>::InvalidVerification,
		);
	})
}
