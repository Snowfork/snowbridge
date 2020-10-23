use crate::mock::{
	child_of_genesis_ethereum_header, new_tester, AccountId, Verifier, MockRuntime, Origin,
};
use crate::sp_api_hidden_includes_decl_storage::hidden_include::{StorageMap, StorageValue};
use frame_support::{assert_err, assert_ok, assert_noop};
use sp_keyring::AccountKeyring as Keyring;

use artemis_core::{Message, VerificationInput};

use crate::{Error, EthereumHeader, EthBestBlock, EthHeaders, EthHeadersByNumber};

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

#[test]
fn it_tracks_highest_difficulty_ethereum_chain() {
	new_tester().execute_with(|| {
		let mut child1 = child_of_genesis_ethereum_header();
		child1.difficulty = 0xbc140caa61087i64.into();
		let child1_hash = child1.compute_hash();
		let mut child2 = child_of_genesis_ethereum_header();
		child2.difficulty = 0x20000.into();

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(Verifier::import_header(Origin::signed(ferdie.clone()), child1));
		assert_ok!(Verifier::import_header(Origin::signed(ferdie.clone()), child2));
	
		let (header_id, highest_difficulty) = EthBestBlock::get();
		assert_eq!(header_id.hash, child1_hash);
		assert_eq!(highest_difficulty, 0xbc140caa61087i64.into());
	});
}

#[test]
fn it_tracks_multiple_unconfirmed_ethereum_forks() {
	new_tester().execute_with(|| {
		let mut child1 = child_of_genesis_ethereum_header();
		child1.number = 1 as u64;
		let child1_hash = child1.compute_hash();
		let mut child2 = child1.clone();
		// make child2 have a different hash to child1
		child2.difficulty = 0x20000.into();
		let child2_hash = child2.compute_hash();

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(Verifier::import_header(Origin::signed(ferdie.clone()), child1));
		assert_ok!(Verifier::import_header(Origin::signed(ferdie.clone()), child2));

		assert!(EthHeaders::<MockRuntime>::contains_key(child1_hash));
		assert!(EthHeaders::<MockRuntime>::contains_key(child2_hash));
		assert_eq!(EthHeadersByNumber::get(1).unwrap(), vec![child1_hash, child2_hash]);
	});
}


#[test]
fn it_imports_ethereum_header_only_once() {
	new_tester().execute_with(|| {
		let child = child_of_genesis_ethereum_header();
		let child_for_reimport = child.clone();

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(Verifier::import_header(Origin::signed(ferdie.clone()), child));
		assert_err!(
			Verifier::import_header(Origin::signed(ferdie.clone()), child_for_reimport),
			"Header can only be imported once",
		);
	});
}

#[test]
fn it_rejects_ethereum_header_not_from_relay() {
	new_tester().execute_with(|| {
		let child = child_of_genesis_ethereum_header();

		let bob: AccountId = Keyring::Bob.into();
		assert_err!(
			Verifier::import_header(Origin::signed(bob), child),
			"Invalid",
		);
	});
}

#[test]
fn it_rejects_ethereum_header_before_parent() {
	new_tester().execute_with(|| {
		let child = child_of_genesis_ethereum_header();
		let mut child_of_child: EthereumHeader = Default::default();
		child_of_child.parent_hash = child.compute_hash();

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_err!(
			Verifier::import_header(Origin::signed(ferdie), child_of_child),
			"Parent header must be imported first",
		);
	});
}
