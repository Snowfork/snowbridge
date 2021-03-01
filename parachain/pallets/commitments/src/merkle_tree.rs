use codec::Encode;
use sp_core::H256;
use sp_trie::TrieConfiguration;

use anyhow;

type Proof = Vec<Vec<u8>>;
type Layout = sp_trie::Layout<sp_core::KeccakHasher>;

pub struct MerkleTree {
	pub root: H256,
	items: Vec<(Vec<u8>, Vec<u8>)>,
	db: sp_trie::MemoryDB<sp_core::KeccakHasher>,
}

impl MerkleTree {
	pub fn new<T: Encode>(items: impl Iterator<Item = T>) -> Self {
		let ordered_items = items
			.map(|x| Encode::encode(&x))
			.enumerate()
			.map(|(i, v)| (Layout::encode_index(i as u32), v))
			.collect::<Vec<(Vec<u8>, Vec<u8>)>>();

		let mut db = sp_trie::MemoryDB::<sp_core::KeccakHasher>::default();
		let mut cb = trie_db::TrieBuilder::new(&mut db);
		let trie_items = ordered_items.clone();
		trie_db::trie_visit::<Layout, _, _, _, _>(trie_items.into_iter(), &mut cb);

		Self {
			root: cb.root.unwrap_or_default(),
			items: ordered_items,
			db,
		}
	}

	#[allow(dead_code)]
	pub fn generate_proof(&self, leaf_index: usize) -> anyhow::Result<Proof> {
		let leaf = self.items.get(leaf_index).cloned().ok_or_else(|| {
			anyhow::format_err!(
				"Leaf index out of bounds: {} vs {}",
				leaf_index,
				self.items.len(),
			)
		})?;
		let proof: Proof =
			sp_trie::generate_trie_proof::<Layout, _, _, _>(&self.db, self.root, vec![&leaf.0])?;
		Ok(proof)
	}
}

// #[allow(dead_code)]
// pub fn generate_merkle_proof<T: Encode>(
//     items: impl Iterator<Item = T>,
//     leaf_index: usize,
// ) -> anyhow::Result<Proof> {
//     let ordered_items = items
//         .map(|x| Encode::encode(&x))
//         .enumerate()
//         .map(|(i, v)| (Layout::encode_index(i as u32), v))
//         .collect::<Vec<(Vec<u8>, Vec<u8>)>>();
//     let leaf = ordered_items.get(leaf_index).cloned().ok_or_else(|| {
//         anyhow::format_err!(
//             "Leaf index out of bounds: {} vs {}",
//             leaf_index,
//             ordered_items.len(),
//         )
//     })?;
//     let mut db = sp_trie::MemoryDB::<sp_core::KeccakHasher>::default();
//     let mut cb = trie_db::TrieBuilder::new(&mut db);
//     trie_db::trie_visit::<Layout, _, _, _, _>(ordered_items.into_iter(), &mut cb);
//     let root = cb.root.unwrap_or_default();
//     let proof: Proof = sp_trie::generate_trie_proof::<Layout, _, _, _>(&db, root, vec![&leaf.0])?;

//     let encoded_proof = proof.encode();

//     println!();
//     println!("Root: {:?}", root);
//     println!("SCALE-encoded proof: 0x{}", hex::encode(encoded_proof));
//     println!("\nLeaf key: 0x{}", hex::encode(&leaf.0));
//     println!("SCALE-encoded leaf value: 0x{}", hex::encode(&leaf.1));
//     println!();

//     Ok(proof)
// }
