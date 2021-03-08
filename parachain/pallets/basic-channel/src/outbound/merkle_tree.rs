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

type Proof = Vec<Vec<u8>>;
type Layout = sp_trie::Layout<KeccakHasher>;

pub struct MerkleTree {
	pub root: H256,
	items: Vec<(Vec<u8>, Vec<u8>)>,
	db: sp_trie::MemoryDB<KeccakHasher>,
}

impl MerkleTree {
	pub fn new<T: Encode>(items: impl Iterator<Item = T>) -> Self {
		let ordered_items = items
			.map(|x| Encode::encode(&x))
			.enumerate()
			.map(|(i, v)| (Layout::encode_index(i as u32), v))
			.collect::<Vec<(Vec<u8>, Vec<u8>)>>();

		let mut db = sp_trie::MemoryDB::<KeccakHasher>::default();
		let mut cb = trie_db::TrieBuilder::new(&mut db);
		let trie_items = ordered_items.clone();
		trie_db::trie_visit::<Layout, _, _, _, _>(trie_items.into_iter(), &mut cb);

		Self {
			root: cb.root.unwrap_or_default(),
			items: ordered_items,
			db,
		}
	}

	// #[allow(dead_code)]
	// pub fn generate_proof(&self, leaf_index: usize) -> anyhow::Result<Proof> {
	// 	let leaf = self
	// 		.items
	// 		.get(leaf_index)
	// 		.cloned()
	// 		.ok_or_else(|| anyhow::format_err!("Leaf index out of bounds"))?;
	// 	let proof: Proof =
	// 		sp_trie::generate_trie_proof::<Layout, _, _, _>(&self.db, self.root, vec![&leaf.0])?;
	// 	Ok(proof)
	// }
}

