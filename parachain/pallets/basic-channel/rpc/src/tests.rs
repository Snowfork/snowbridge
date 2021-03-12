use super::*;
use codec::Encode;
use sp_core::{H256, offchain::{storage::InMemOffchainStorage}};
use assert_matches::assert_matches;
use hex_literal::hex;

type AccountId = u64;

#[test]
fn local_storage_should_work() {
	let storage = InMemOffchainStorage::default();
	let channel = BasicChannel::<_, AccountId>::new(storage, DenyUnsafe::No, b"testing");
	let root = H256::from_slice(&hex!["aaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbbcccccccccccccccdddddddddddddddd"]); // = Bytes(b"offchain_storage".to_vec());
	let key = offchain_key(channel.indexing_prefix, root);
	let account_id = 1234u64;

	let value = CommitmentData{
		messages: Vec::new(),
		subcommitments: vec![(account_id, b"some_data".to_vec())],
	};
	let value = value.encode();

	channel.storage.write().set(sp_offchain::STORAGE_PREFIX, &*key, &*value.clone());

	assert!(channel.get_merkle_proofs(root).is_ok());

	assert_matches!(
		channel.get_merkle_proofs(root),
		Ok(ref proofs) if proofs[0].0 == account_id
	);

	let root2 = H256::from_slice(&hex!["0aaaaaaaaaaaaaaabbbbbbbbbbbbbbbbbcccccccccccccccdddddddddddddddd"]); // = Bytes(b"offchain_storage".to_vec());
	assert!(!channel.get_merkle_proofs(root2).is_ok());
}

#[test]
fn offchain_calls_considered_unsafe() {
	let storage = InMemOffchainStorage::default();
	let channel = BasicChannel::<_, AccountId>::new(storage, DenyUnsafe::Yes, b"testing");
	let root = H256::from_slice(&hex!["aaaaaaaaaaaaaaaabbbbbbbbbbbbbbbbbcccccccccccccccdddddddddddddddd"]); // = Bytes(b"offchain_storage".to_vec());
	let key = offchain_key(channel.indexing_prefix, root);
	let account_id = 1234u64;

	let value = CommitmentData{
		messages: Vec::new(),
		subcommitments: vec![(account_id, b"some_data".to_vec())],
	};
	let value = value.encode();

	channel.storage.write().set(sp_offchain::STORAGE_PREFIX, &*key, &*value.clone());

	assert_matches!(
		channel.get_merkle_proofs(root),
	    	Err(jsonrpc_core::Error{..})
	);
}
