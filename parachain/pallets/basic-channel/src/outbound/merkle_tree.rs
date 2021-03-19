use sp_core::{Hasher, H256};
use sp_std::prelude::*;
use sp_trie::TrieConfiguration;
use hash256_std_hasher::Hash256StdHasher;

/// Concrete implementation of Hasher using Keccak 256-bit hashes
#[derive(Debug)]
pub struct KeccakHasher;

impl Hasher for KeccakHasher {
	type Out = H256;
	type StdHasher = Hash256StdHasher;
	const LENGTH: usize = 32;

	fn hash(x: &[u8]) -> Self::Out {
		sp_io::hashing::keccak_256(x).into()
	}
}

#[derive(Debug)]
pub enum Error {
    MerkleProofError,
}

type Layout = sp_trie::Layout<KeccakHasher>;
type EnumeratedItems = Vec<(Vec<u8>, Vec<u8>)>;

pub fn generate_merkle_root(items: impl Iterator<Item = Vec<u8>>) -> H256 {
	let mut db = sp_trie::MemoryDB::<KeccakHasher>::default();
	let mut cb = trie_db::TrieBuilder::new(&mut db);
	trie_db::trie_visit::<Layout, _, _, _, _>(items
		.enumerate()
		.map(|(i, v)| (Layout::encode_index(i as u32), v)), &mut cb);

	cb.root.unwrap_or_default()
}

pub fn generate_merkle_proofs(encoded_items: impl Iterator<Item = Vec<u8>>) -> Result<Vec<Vec<u8>>, Error> {
	let enumerated_items = encoded_items
		.enumerate()
		.map(|(i, v)| (Layout::encode_index(i as u32), v))
		.collect::<EnumeratedItems>();

	let mut db = sp_trie::MemoryDB::<KeccakHasher>::default();
	let mut cb = trie_db::TrieBuilder::new(&mut db);
	trie_db::trie_visit::<Layout, _, _, _, _>(enumerated_items.iter().cloned(), &mut cb);
	let root = cb.root.unwrap_or_default();
	let proofs: Result<Vec<Vec<u8>>, _> = enumerated_items
		.iter()
		.map( |(idx, _)| {
			sp_trie::generate_trie_proof::<Layout, _, _, _>(&db, root, vec![&idx])
				.map(|v| v[0].clone())
		})
		.collect();
	proofs.or(Err(Error::MerkleProofError{}))
}

#[test]
fn merkle_proofs_should_work() {
	let items = vec![
		vec![1,1,1,1u8],
		vec![2,2,2,2u8],
		vec![3,3,3,3u8],
	];

	let root = generate_merkle_root(items.clone().into_iter());
	let proofs = generate_merkle_proofs(items.clone().into_iter()).map_err(|_| Error::MerkleProofError{}).unwrap();

	for i in 0..3 {
		assert!(sp_trie::verify_trie_proof::<Layout, _, _, _>(
				&root,
				&[proofs[i].clone()],
				vec![(Layout::encode_index(i as u32), Some(items[i].clone()))].iter(),
			).is_ok());
	};
}
