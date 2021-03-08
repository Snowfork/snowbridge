use codec::Encode;
use sp_core::{Hasher, H256};
use sp_std::prelude::*;
use sp_trie::TrieConfiguration;
use hash256_std_hasher::Hash256StdHasher;

//use hash256_std_hasher::Hash256StdHasher;

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

pub struct MerkleProofError;
pub type EncodedItems = Vec<(Vec<u8>, Vec<u8>)>;

type Layout = sp_trie::Layout<KeccakHasher>;

pub fn generate_merkle_root<T: Encode>(items: impl Iterator<Item = T>) -> H256 {
	let encoded_items = items
		.map(|x| Encode::encode(&x))
		.enumerate()
		.map(|(i, v)| (Layout::encode_index(i as u32), v))
		.collect::<Vec<(Vec<u8>, Vec<u8>)>>();

	let mut db = sp_trie::MemoryDB::<KeccakHasher>::default();
	let mut cb = trie_db::TrieBuilder::new(&mut db);
	trie_db::trie_visit::<Layout, _, _, _, _>(encoded_items.into_iter(), &mut cb);

	cb.root.unwrap_or_default()
}

pub fn generate_merkle_proofs(encoded_items: EncodedItems) -> Result<Vec<Vec<u8>>, MerkleProofError> {
	let leafs = encoded_items.iter().map(|(k, _)| k.clone()).collect::<Vec<Vec<u8>>>();
	let mut db = sp_trie::MemoryDB::<KeccakHasher>::default();
	let mut cb = trie_db::TrieBuilder::new(&mut db);
	trie_db::trie_visit::<Layout, _, _, _, _>(encoded_items.into_iter(), &mut cb);
	let root = cb.root.unwrap_or_default();
 	let proofs = sp_trie::generate_trie_proof::<Layout, _, _, _>(
		&db,
		root,
		leafs.iter().collect::<Vec<&Vec<u8>>>());
    	proofs.map_err(|_| MerkleProofError{})
}

