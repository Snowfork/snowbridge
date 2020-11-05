#![cfg(test)]

use crate::mock::{
	child_of_genesis_ethereum_header, ethereum_header_from_file,
	ethereum_header_proof_from_file, new_tester, new_tester_with_config,
	AccountId, Verifier, MockRuntime, Origin,
};
use crate::sp_api_hidden_includes_decl_storage::hidden_include::{StorageMap, StorageValue};
use frame_support::{assert_err, assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::DispatchError;
use crate::{BestBlock, Error, EthereumHeader, GenesisConfig, Headers, HeadersByNumber};

#[test]
fn it_tracks_highest_difficulty_ethereum_chain() {
	new_tester().execute_with(|| {
		let mut child1 = child_of_genesis_ethereum_header();
		child1.difficulty = 0xbc140caa61087i64.into();
		let child1_hash = child1.compute_hash();
		let mut child2 = child_of_genesis_ethereum_header();
		child2.difficulty = 0x20000.into();

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			child1,
			Default::default(),
		));
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			child2,
			Default::default(),
		));
	
		let (header_id, highest_difficulty) = BestBlock::get();
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
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			child1,
			Default::default(),
		));
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			child2,
			Default::default(),
		));

		assert!(Headers::<MockRuntime>::contains_key(child1_hash));
		assert!(Headers::<MockRuntime>::contains_key(child2_hash));
		assert_eq!(HeadersByNumber::get(1).unwrap(), vec![child1_hash, child2_hash]);
	});
}


#[test]
fn it_imports_ethereum_header_only_once() {
	new_tester().execute_with(|| {
		let child = child_of_genesis_ethereum_header();
		let child_for_reimport = child.clone();

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			child,
			Default::default(),
		));
		assert_err!(
			Verifier::import_header(
				Origin::signed(ferdie.clone()),
				child_for_reimport,
				Default::default(),
			),
			Error::<MockRuntime>::DuplicateHeader,
		);
	});
}

#[test]
fn it_rejects_unsigned_ethereum_header() {
	new_tester().execute_with(|| {
		let child = child_of_genesis_ethereum_header();
		assert_err!(
			Verifier::import_header(Origin::none(), child, Default::default()),
			DispatchError::BadOrigin,
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
			Verifier::import_header(
				Origin::signed(ferdie),
				child_of_child,
				Default::default(),
			),
			Error::<MockRuntime>::MissingParentHeader,
		);
	});
}

#[test]
fn it_validates_proof_of_work() {
	new_tester_with_config(GenesisConfig {
		initial_header: ethereum_header_from_file(11090290),
		initial_difficulty: 0.into(),
		verify_pow: true,
	}).execute_with(|| {
		let header1 = ethereum_header_from_file(11090291);
		let header1_proof = ethereum_header_proof_from_file(11090291);
		let header2 = ethereum_header_from_file(11090292);
		let header2_proof = ethereum_header_proof_from_file(11090292);

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			header1,
			header1_proof,
		));
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie),
			header2,
			header2_proof,
		));
	});
}
