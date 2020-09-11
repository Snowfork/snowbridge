use crate::mock::{new_tester, AccountId, Verifier, MockRuntime};
use frame_support::{assert_ok, assert_noop};
use sp_keyring::AccountKeyring as Keyring;

use artemis_core::{Message, VerificationInput};

use crate::Error;

#[test]
fn it_verifies_different_messages() {
	new_tester().execute_with(|| {
		let ferdie: AccountId = Keyring::Ferdie.into();
		let app_id: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
		let message  = Message {
			payload: [0, 1, 3].to_vec(),
			verification: VerificationInput::Basic {block_number: 938, event_index: 4}
		};
		assert_ok!(Verifier::do_verify(ferdie.clone(), app_id, &message));

		let message  = Message {
			payload: [0, 2, 3].to_vec(),
			verification: VerificationInput::Basic {block_number: 970, event_index: 3}
		};
		assert_ok!(Verifier::do_verify(ferdie.clone(), app_id, &message));

		let message  = Message {
			payload: [7, 8, 9].to_vec(),
			verification: VerificationInput::Basic {block_number: 981, event_index: 0}
		};
		assert_ok!(Verifier::do_verify(ferdie, app_id, &message));
	});
}

#[test]
fn it_checks_unauthorized_sender() {
	new_tester().execute_with(|| {
		let bob: AccountId = Keyring::Bob.into();
		let app_id: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
		let message  = Message {
			payload: [0, 1, 3].to_vec(),
			verification: VerificationInput::Basic {block_number: 938, event_index: 4}
		};

		assert_noop!(
			Verifier::do_verify(bob, app_id, &message),
			Error::<MockRuntime>::Invalid
		);
	});
}

#[test]
fn it_checks_for_replayed_messages() {
	new_tester().execute_with(|| {
		let ferdie: AccountId = Keyring::Ferdie.into();
		let app_id: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
		let message  = Message {
			payload: [0, 1, 3].to_vec(),
			verification: VerificationInput::Basic {block_number: 938, event_index: 4}
		};
		assert_ok!(Verifier::do_verify(ferdie.clone(), app_id, &message));
		assert_noop!(
			Verifier::do_verify(ferdie, app_id, &message),
			Error::<MockRuntime>::Invalid
		);
	});
}
