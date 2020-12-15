use artemis_core::{VerificationOutput, Verifier as VerifierTrait};
use crate::mock::{
	child_of_genesis_ethereum_header, child_of_header, ethereum_header_from_file,
	ethereum_header_proof_from_file, genesis_ethereum_block_hash, message_with_receipt_proof,
	new_tester, new_tester_with_config, receipt_root_and_proof, AccountId, Verifier,
	VerifierWithPoW, MockRuntime, MockRuntimeWithPoW, Origin,
};
use crate::sp_api_hidden_includes_decl_storage::hidden_include::{StorageMap, StorageValue};
use frame_support::{assert_err, assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::DispatchError;
use crate::{
	BestBlock, Error, EthereumHeader, FinalizedBlock, GenesisConfig, Headers,
	HeadersByNumber, PruningRange,
};

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
fn it_tracks_multiple_unfinalized_ethereum_forks() {
	new_tester().execute_with(|| {
		let child1 = child_of_genesis_ethereum_header();
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
fn it_tracks_only_one_finalized_ethereum_fork() {
	new_tester().execute_with(|| {
		let block1 = child_of_genesis_ethereum_header();
		let block1_hash = block1.compute_hash();
		let block2 = child_of_header(&block1);
		let block3 = child_of_header(&block2);
		let block3_hash = block3.compute_hash();
		let mut block4 = child_of_genesis_ethereum_header();
		block4.difficulty = 2.into();
		let mut block5 = child_of_header(&block4);
		block5.difficulty = 3.into();
		let mut block6 = child_of_genesis_ethereum_header();
		block6.difficulty = 5.into();

		// Initial chain:
		//   B0
		//   |  \
		//   B1  B4
		//   |
		//   B2
		//   |
		//   B3
		let ferdie: AccountId = Keyring::Ferdie.into();
		for header in vec![block1, block4, block2, block3].into_iter() {
			assert_ok!(Verifier::import_header(
				Origin::signed(ferdie.clone()),
				header,
				Default::default(),
			));
		}
		// Relies on DescendantsUntilFinalized = 2
		assert_eq!(FinalizedBlock::get().hash, block1_hash);
		assert_eq!(BestBlock::get().0.hash, block3_hash);

		// With invalid forks (invalid since B1 is final):
		//       B0
		//     / | \
		//   B6  B1  B4
		//       |    \
		//       B2    B5
		//       |
		//       B3
		assert_err!(
			Verifier::import_header(
				Origin::signed(ferdie.clone()),
				block5,
				Default::default(),
			),
			Error::<MockRuntime>::HeaderOnStaleFork,
		);
		assert_err!(
			Verifier::import_header(
				Origin::signed(ferdie.clone()),
				block6,
				Default::default(),
			),
			Error::<MockRuntime>::AncientHeader,
		);
	});
}

#[test]
fn it_prunes_ethereum_headers_correctly() {
	new_tester().execute_with(|| {
		let block1 = child_of_genesis_ethereum_header();
		let block1_hash = block1.compute_hash();
		let block2 = child_of_header(&block1);
		let block2_hash = block2.compute_hash();
		let block3 = child_of_header(&block2);
		let block3_hash = block3.compute_hash();
		let mut block4 = child_of_genesis_ethereum_header();
		block4.difficulty = 2.into();
		let block4_hash = block4.compute_hash();

		// Initial chain:
		//   B0
		//   |  \
		//   B1  B4
		//   |
		//   B2
		//   |
		//   B3
		let ferdie: AccountId = Keyring::Ferdie.into();
		for header in vec![block1, block4, block2, block3].into_iter() {
			assert_ok!(Verifier::import_header(
				Origin::signed(ferdie.clone()),
				header,
				Default::default(),
			));
		}

		// Prune genesis block
		let new_range = Verifier::prune_header_range(
			&PruningRange { oldest_unpruned_block: 0, oldest_block_to_keep: 1 },
			2,
			1,
		);
		assert_eq!(
			new_range,
			PruningRange { oldest_unpruned_block: 1, oldest_block_to_keep: 1 },
		);
		assert!(!Headers::<MockRuntime>::contains_key(genesis_ethereum_block_hash()));
		assert!(!HeadersByNumber::contains_key(0));

		// Prune next block (B1)
		let new_range = Verifier::prune_header_range(
			&PruningRange { oldest_unpruned_block: 1, oldest_block_to_keep: 1 },
			1,
			2,
		);
		assert_eq!(
			new_range,
			PruningRange { oldest_unpruned_block: 1, oldest_block_to_keep: 2 },
		);
		assert!(!Headers::<MockRuntime>::contains_key(block1_hash));
		assert!(Headers::<MockRuntime>::contains_key(block4_hash));
		assert_eq!(HeadersByNumber::get(1).unwrap(), vec![block4_hash]);

		// Prune next two blocks (B4, B2)
		let new_range = Verifier::prune_header_range(
			&PruningRange { oldest_unpruned_block: 1, oldest_block_to_keep: 2 },
			2,
			4,
		);
		assert_eq!(
			new_range,
			PruningRange { oldest_unpruned_block: 3, oldest_block_to_keep: 4 },
		);
		assert!(!Headers::<MockRuntime>::contains_key(block4_hash));
		assert!(!HeadersByNumber::contains_key(1));
		assert!(!Headers::<MockRuntime>::contains_key(block2_hash));
		assert!(!HeadersByNumber::contains_key(2));

		// Finally, we're left with B3
		assert!(Headers::<MockRuntime>::contains_key(block3_hash));
		assert_eq!(HeadersByNumber::get(3).unwrap(), vec![block3_hash]);
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
	new_tester_with_config::<MockRuntimeWithPoW>(GenesisConfig {
			initial_header: ethereum_header_from_file(11090290),
			initial_difficulty: 0.into(),
	}).execute_with(|| {
		let header1 = ethereum_header_from_file(11090291);
		let header1_proof = ethereum_header_proof_from_file(11090291);
		let header2 = ethereum_header_from_file(11090292);

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(VerifierWithPoW::import_header(
			Origin::signed(ferdie.clone()),
			header1,
			header1_proof,
		));
		assert_err!(
			VerifierWithPoW::import_header(
				Origin::signed(ferdie),
				header2,
				Default::default(),
			),
			Error::<MockRuntimeWithPoW>::InvalidHeader,
		);
	});
}

#[test]
fn it_confirms_receipt_inclusion_in_finalized_header() {
	let (receipts_root, receipt_proof) = receipt_root_and_proof();
	let mut finalized_header: EthereumHeader = Default::default();
	finalized_header.receipts_root = receipts_root;
	let finalized_header_hash = finalized_header.compute_hash();

	new_tester_with_config::<MockRuntime>(GenesisConfig {
		initial_header: finalized_header,
		initial_difficulty: 0.into(),
	}).execute_with(|| {
		let output = Verifier::verify(
			Default::default(),
			Default::default(),
			&message_with_receipt_proof(finalized_header_hash, receipt_proof),
		);
		assert_ok!(&output);
		assert!(match output.unwrap() {
			VerificationOutput::Receipt(receipt) => true,
			_ => false,
		});
	});
}

#[test]
fn it_denies_receipt_inclusion_for_invalid_proof() {
	new_tester().execute_with(|| {
		let (_, receipt_proof) = receipt_root_and_proof();
		assert_err!(
			Verifier::verify(
				Default::default(),
				Default::default(),
				&message_with_receipt_proof(genesis_ethereum_block_hash(), receipt_proof),
			),
			Error::<MockRuntime>::InvalidProof,
		);
	});
}

#[test]
fn it_denies_receipt_inclusion_for_invalid_header() {
	new_tester().execute_with(|| {
		let (receipts_root, receipt_proof) = receipt_root_and_proof();
		let mut block1 = child_of_genesis_ethereum_header();
		block1.receipts_root = receipts_root;
		let block1_hash = block1.compute_hash();
		let mut block1_alt = child_of_genesis_ethereum_header();
		block1_alt.receipts_root = receipts_root;
		block1_alt.difficulty = 2.into();
		let block1_alt_hash = block1_alt.compute_hash();
		let block2_alt = child_of_header(&block1_alt);
		let block3_alt = child_of_header(&block2_alt);
		let block4_alt = child_of_header(&block3_alt);

		// Header hasn't been imported yet
		assert_err!(
			Verifier::verify(
				Default::default(),
				Default::default(),
				&message_with_receipt_proof(block1_hash, receipt_proof.clone()),
			),
			Error::<MockRuntime>::MissingHeader,
		);

		let ferdie: AccountId = Keyring::Ferdie.into();
		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			block1,
			Default::default(),
		));

		// Header has been imported but not finalized
		assert_err!(
			Verifier::verify(
				Default::default(),
				Default::default(),
				&message_with_receipt_proof(block1_hash, receipt_proof.clone()),
			),
			Error::<MockRuntime>::HeaderNotFinalized,
		);

		// With alternate fork:
		//   B0
		//   |  \
		//   B1  B1_ALT
		//        \
		//         B2_ALT
		//          \
		//           B3_ALT
		for header in vec![block1_alt, block2_alt, block3_alt].into_iter() {
			assert_ok!(Verifier::import_header(
				Origin::signed(ferdie.clone()),
				header,
				Default::default(),
			));
		}
		assert_eq!(FinalizedBlock::get().hash, block1_alt_hash);

		// A finalized header at this height exists, but it's not block1
		assert_err!(
			Verifier::verify(
				Default::default(),
				Default::default(),
				&message_with_receipt_proof(block1_hash, receipt_proof.clone()),
			),
			Error::<MockRuntime>::HeaderOnStaleFork,
		);

		assert_ok!(Verifier::import_header(
			Origin::signed(ferdie.clone()),
			block4_alt,
			Default::default(),
		));

		// A finalized header at a newer height exists, but block1 isn't its ancestor
		assert_err!(
			Verifier::verify(
				Default::default(),
				Default::default(),
				&message_with_receipt_proof(block1_hash, receipt_proof.clone()),
			),
			Error::<MockRuntime>::HeaderOnStaleFork,
		);
		// Verification works for an ancestor of the finalized header
		assert_ok!(Verifier::verify(
			Default::default(),
			Default::default(),
			&message_with_receipt_proof(block1_alt_hash, receipt_proof.clone()),
		));
	});
}
