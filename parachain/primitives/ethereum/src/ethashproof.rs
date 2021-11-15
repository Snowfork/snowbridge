use codec::{Decode, Encode};
use ethereum_types::{H128, H256, H512, H64};
use sp_io::hashing::{keccak_256, keccak_512, sha2_256};
use sp_runtime::{scale_info::TypeInfo, RuntimeDebug};
use sp_std::{cell::RefCell, collections::btree_map::BTreeMap, prelude::*};

pub use crate::ethashdata::{DAGS_MERKLE_ROOTS, DAGS_START_EPOCH};

/// Ethash Params. See https://eth.wiki/en/concepts/ethash/ethash
/// Blocks per epoch
const EPOCH_LENGTH: u64 = 30000;
/// Width of mix
const MIX_BYTES: usize = 128;
/// Hash length in bytes
const HASH_BYTES: usize = 64;
/// Numver of accesses in hashimoto loop
const ACCESSES: usize = 64;

#[derive(Default, Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct DoubleNodeWithMerkleProof {
	pub dag_nodes: [H512; 2],
	pub proof: Vec<H128>,
}

impl DoubleNodeWithMerkleProof {
	pub fn from_values(dag_nodes: [H512; 2], proof: Vec<H128>) -> Self {
		Self { dag_nodes, proof }
	}

	fn truncate_to_h128(arr: H256) -> H128 {
		let mut data = [0u8; 16];
		data.copy_from_slice(&(arr.0)[16..]);
		H128(data.into())
	}

	fn hash_h128(l: H128, r: H128) -> H128 {
		let mut data = [0u8; 64];
		data[16..32].copy_from_slice(&(l.0));
		data[48..64].copy_from_slice(&(r.0));
		Self::truncate_to_h128(sha2_256(&data).into())
	}

	pub fn apply_merkle_proof(&self, index: u64) -> Result<H128, &'static str> {
		let mut data = [0u8; 128];
		data[..64].copy_from_slice(&(self.dag_nodes[0].0));
		data[64..].copy_from_slice(&(self.dag_nodes[1].0));

		let mut leaf = Self::truncate_to_h128(sha2_256(&data).into());

		for i in 0..self.proof.len() {
			let index_shifted = index.checked_shr(i as u32).ok_or("Failed to shift index")?;
			if index_shifted % 2 == 0 {
				leaf = Self::hash_h128(leaf, self.proof[i]);
			} else {
				leaf = Self::hash_h128(self.proof[i], leaf);
			}
		}
		Ok(leaf)
	}
}

/// A wrapper around ethash::make_cache with LRU caching. Use this to retrieve
/// DAG cache data for a given epoch.
pub struct EthashCache {
	/// Maximum number of DAG caches we'll store at a time
	max_capacity: usize,
	/// Most recently accessed DAG caches, stored as epoch => cache data
	caches_by_epoch: BTreeMap<u64, Vec<u8>>,
	/// (timestamp, epoch) of the most recently accessed caches, ordered from least to most recent
	recently_accessed_epochs: Vec<(u64, u64)>,
	/// Cache data generator
	cache_gen_fn: fn(usize) -> Vec<u8>,
}

impl EthashCache {
	pub fn new(max: usize) -> EthashCache {
		assert!(max > 0);
		EthashCache {
			max_capacity: max,
			caches_by_epoch: BTreeMap::new(),
			recently_accessed_epochs: Vec::with_capacity(max),
			cache_gen_fn: Self::get_cache_for_epoch,
		}
	}

	/// For tests to override the cache data generator
	pub fn with_generator(max: usize, cache_gen_fn: fn(usize) -> Vec<u8>) -> EthashCache {
		let mut cache = EthashCache::new(max);
		cache.cache_gen_fn = cache_gen_fn;
		cache
	}

	pub fn get(&mut self, epoch: u64, timestamp: u64) -> &Vec<u8> {
		if self.caches_by_epoch.contains_key(&epoch) {
			let (ref mut t, _e) = self
				.recently_accessed_epochs
				.iter_mut()
				.find(|&&mut pair| pair.1 == epoch)
				.unwrap();
			*t = timestamp;
		} else {
			if self.recently_accessed_epochs.len() == self.max_capacity {
				let (ref mut t, ref mut e) = self.recently_accessed_epochs.first_mut().unwrap();
				self.caches_by_epoch.remove(e);
				*t = timestamp;
				*e = epoch;
			} else {
				self.recently_accessed_epochs.push((timestamp, epoch));
			}
			let cache_gen_fn = self.cache_gen_fn;
			self.caches_by_epoch.insert(epoch, cache_gen_fn(epoch as usize));
		}

		self.recently_accessed_epochs.sort();
		self.caches_by_epoch.get(&epoch).unwrap()
	}

	fn get_cache_for_epoch(epoch: usize) -> Vec<u8> {
		let seed = ethash::get_seedhash(epoch);
		let cache_size = ethash::get_cache_size(epoch);
		let mut data = vec![0; cache_size];
		ethash::make_cache(data.as_mut_slice(), seed);
		data
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
	// Epoch doesn't map to the range in DAGS_MERKLE_ROOTS
	EpochOutOfRange,
	// The merkle proof could not be verified
	InvalidMerkleProof,
	// The number of nodes with proof don't match the expected number of DAG nodes
	UnexpectedNumberOfNodes,
}

pub struct EthashProver {
	/// A LRU cache of DAG caches
	dags_cache: Option<EthashCache>,
}

impl EthashProver {
	pub fn new() -> Self {
		Self { dags_cache: None }
	}

	pub fn with_hashimoto_light(max_cache_entries: usize) -> Self {
		Self { dags_cache: Some(EthashCache::new(max_cache_entries)) }
	}

	fn dag_merkle_root(&self, epoch: u64) -> Option<H128> {
		DAGS_MERKLE_ROOTS
			.get((epoch - DAGS_START_EPOCH) as usize)
			.map(|x| H128::from(x))
	}

	// Adapted fro https://github.com/near/rainbow-bridge/blob/3fcdfbc6c0011f0e1507956a81c820616fb963b4/contracts/near/eth-client/src/lib.rs#L363
	pub fn hashimoto_merkle(
		&self,
		header_hash: H256,
		nonce: H64,
		header_number: u64,
		nodes: &[DoubleNodeWithMerkleProof],
	) -> Result<(H256, H256), Error> {
		// Check that we have the expected number of nodes with proofs
		const MIXHASHES: usize = MIX_BYTES / HASH_BYTES;
		if nodes.len() != MIXHASHES * ACCESSES / 2 {
			return Err(Error::UnexpectedNumberOfNodes)
		}

		let epoch = header_number / EPOCH_LENGTH;
		// Reuse single Merkle root across all the proofs
		let merkle_root = self.dag_merkle_root(epoch).ok_or(Error::EpochOutOfRange)?;
		let full_size = ethash::get_full_size(epoch as usize);

		// Boxed index since ethash::hashimoto gets Fn, but not FnMut
		let index = RefCell::new(0);
		// Flag for whether the proof is valid
		let success = RefCell::new(true);

		let results = ethash::hashimoto_with_hasher(
			header_hash,
			nonce,
			full_size,
			|offset| {
				let idx = *index.borrow_mut();
				*index.borrow_mut() += 1;

				// Each two nodes are packed into single 128 bytes with Merkle proof
				let node = &nodes[idx / 2];
				if idx % 2 == 0 {
					// Divide by 2 to adjust offset for 64-byte words instead of 128-byte
					if let Ok(computed_root) = node.apply_merkle_proof((offset / 2) as u64) {
						if merkle_root != computed_root {
							success.replace(false);
						}
					} else {
						success.replace(false);
					}
				};

				// Reverse each 32 bytes for ETHASH compatibility
				let mut data = node.dag_nodes[idx % 2].0;
				data[..32].reverse();
				data[32..].reverse();
				data.into()
			},
			keccak_256,
			keccak_512,
		);

		match success.into_inner() {
			true => Ok(results),
			false => Err(Error::InvalidMerkleProof),
		}
	}

	pub fn hashimoto_light(
		&mut self,
		header_hash: H256,
		nonce: H64,
		header_number: u64,
	) -> (H256, H256) {
		let epoch = header_number / EPOCH_LENGTH;
		let cache = match self.dags_cache {
			Some(ref mut c) => c.get(epoch, header_number),
			None => panic!("EthashProver wasn't configured with hashimoto light cache"),
		};
		let full_size = ethash::get_full_size(epoch as usize);
		return ethash::hashimoto_light(header_hash, nonce, full_size, cache.as_slice())
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use hex_literal::hex;
	use rand::Rng;
	use snowbridge_testutils::BlockWithProofs;
	use std::path::PathBuf;

	fn fixture_path(name: &str) -> PathBuf {
		[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name].iter().collect()
	}

	#[test]
	fn cache_removes_oldest_at_capacity() {
		let mut cache = EthashCache::with_generator(1, |_| Vec::new());
		cache.get(10, 1);
		cache.get(20, 2);
		assert_eq!(cache.caches_by_epoch.len(), 1);
		assert_eq!(cache.recently_accessed_epochs.len(), 1);
		assert!(cache.caches_by_epoch.contains_key(&20));
		assert_eq!(cache.recently_accessed_epochs[0].1, 20);
	}

	#[test]
	fn cache_retrieves_existing_and_updates_timestamp() {
		let mut cache = EthashCache::with_generator(2, |_| {
			let mut rng = rand::thread_rng();
			vec![rng.gen()]
		});
		let data1 = cache.get(10, 1).clone();
		let data2 = cache.get(10, 2);
		assert_eq!(data1, *data2);
		assert_eq!(cache.caches_by_epoch.len(), 1);
		assert_eq!(cache.recently_accessed_epochs.len(), 1);
		assert_eq!(cache.recently_accessed_epochs[0].0, 2);
	}

	#[cfg(feature = "expensive_tests")]
	#[test]
	fn hashimoto_light_is_correct_block_11090290() {
		// https://etherscan.io/block/11090290
		let header_partial_hash: H256 =
			hex!("932c22685fd0fb6a1b5f6b70d2ebf4bfd9f3b4f15eb706450a9b050ec0f151c9").into();
		let header_number: u64 = 11090290;
		let header_nonce: H64 = hex!("6935bbe7b63c4f8e").into();
		let header_mix_hash: H256 =
			hex!("be3adfb0087be62b28b716e2cdf3c79329df5caa04c9eee035d35b5d52102815").into();

		let mut prover = EthashProver::with_hashimoto_light(1);
		let (mix_hash, _) =
			prover.hashimoto_light(header_partial_hash, header_nonce, header_number);
		assert_eq!(mix_hash, header_mix_hash);
	}

	#[test]
	fn hashimoto_merkle_is_correct_block_3() {
		// https://etherscan.io/block/3
		let block_with_proofs = BlockWithProofs::from_file(&fixture_path("3.json"));
		let header_partial_hash: H256 =
			hex!("481f55e00fd23652cb45ffba86a08b8d497f3b18cc2c0f14cbeb178b4c386e10").into();
		let header_number: u64 = 3;
		let header_nonce: H64 = hex!("2e9344e0cbde83ce").into();
		let header_mix_hash: H256 =
			hex!("65e12eec23fe6555e6bcdb47aa25269ae106e5f16b54e1e92dcee25e1c8ad037").into();

		let (mix_hash, _) = EthashProver::new()
			.hashimoto_merkle(
				header_partial_hash,
				header_nonce,
				header_number,
				&(block_with_proofs
					.to_double_node_with_merkle_proof_vec(DoubleNodeWithMerkleProof::from_values)),
			)
			.unwrap();
		assert_eq!(header_mix_hash, mix_hash);
	}

	#[test]
	fn hashimoto_merkle_is_correct_block_11090290() {
		// https://etherscan.io/block/11090290
		let block_with_proofs = BlockWithProofs::from_file(&fixture_path("11090290.json"));
		let header_partial_hash: H256 =
			hex!("932c22685fd0fb6a1b5f6b70d2ebf4bfd9f3b4f15eb706450a9b050ec0f151c9").into();
		let header_number: u64 = 11090290;
		let header_nonce: H64 = hex!("6935bbe7b63c4f8e").into();
		let header_mix_hash: H256 =
			hex!("be3adfb0087be62b28b716e2cdf3c79329df5caa04c9eee035d35b5d52102815").into();

		let (mix_hash, _) = EthashProver::new()
			.hashimoto_merkle(
				header_partial_hash,
				header_nonce,
				header_number,
				&(block_with_proofs
					.to_double_node_with_merkle_proof_vec(DoubleNodeWithMerkleProof::from_values)),
			)
			.unwrap();
		assert_eq!(header_mix_hash, mix_hash);
	}

	#[test]
	fn hashimoto_merkle_is_correct_block_11550000() {
		// https://etherscan.io/block/11550000
		let block_with_proofs = BlockWithProofs::from_file(&fixture_path("11550000.json"));
		let header_partial_hash: H256 =
			hex!("7bc3c6073de95a429663dcc4c25f9559cfe1947142d111d91d1e09120c68847e").into();
		let header_number: u64 = 11550000;
		let header_nonce: H64 = hex!("8ae5c070892cb70c").into();
		let header_mix_hash: H256 =
			hex!("0363fe29940988ca043713840ac911b32f2acb4d010e55963f2d201d79f9ab57").into();

		let (mix_hash, _) = EthashProver::new()
			.hashimoto_merkle(
				header_partial_hash,
				header_nonce,
				header_number,
				&(block_with_proofs
					.to_double_node_with_merkle_proof_vec(DoubleNodeWithMerkleProof::from_values)),
			)
			.unwrap();
		assert_eq!(header_mix_hash, mix_hash);
	}

	#[test]
	fn hashimoto_merkle_returns_err_for_invalid_data() {
		let block_with_proofs = BlockWithProofs::from_file(&fixture_path("3.json"));
		let header_partial_hash: H256 =
			hex!("481f55e00fd23652cb45ffba86a08b8d497f3b18cc2c0f14cbeb178b4c386e10").into();
		let header_number: u64 = 3;
		let header_nonce: H64 = hex!("2e9344e0cbde83ce").into();
		let mut proofs = block_with_proofs
			.to_double_node_with_merkle_proof_vec(DoubleNodeWithMerkleProof::from_values);
		let prover = EthashProver::new();

		assert_eq!(
			prover.hashimoto_merkle(header_partial_hash, header_nonce, 30000000, &proofs),
			Err(Error::EpochOutOfRange),
		);
		assert_eq!(
			prover.hashimoto_merkle(
				header_partial_hash,
				header_nonce,
				header_number,
				Default::default()
			),
			Err(Error::UnexpectedNumberOfNodes),
		);
		proofs[0].proof[0] = H128::zero();
		assert_eq!(
			prover.hashimoto_merkle(header_partial_hash, header_nonce, header_number, &proofs),
			Err(Error::InvalidMerkleProof),
		);
	}

	extern crate wasm_bindgen_test;

	use wasm_bindgen_test::*;

	#[wasm_bindgen_test]
	fn hashimoto_breakdown_11550000_wasm() {
		let header_hash: H256 =
			hex!("7bc3c6073de95a429663dcc4c25f9559cfe1947142d111d91d1e09120c68847e").into();
		let nonce: H64 = hex!("8ae5c070892cb70c").into();
		let epoch = 11550000 / EPOCH_LENGTH;
		let full_size = ethash::get_full_size(epoch as usize);

		let index = RefCell::new(0);

		ethash::hashimoto_with_hasher(
			header_hash,
			nonce,
			full_size,
			|offset| {
				let idx = *index.borrow_mut();
				*index.borrow_mut() += 1;
				assert_eq!(offset, DAG_INDICES_11550000[idx]);
				DAG_NODES_11550000[idx].into()
			},
			keccak_256,
			keccak_512,
		);
	}

	const DAG_INDICES_11550000: [usize; 128] = [
		17124670, 17124671, 8406228, 8406229, 62843670, 62843671, 51608408, 51608409, 13778580,
		13778581, 32065328, 32065329, 34759690, 34759691, 3535582, 3535583, 9322842, 9322843,
		35441302, 35441303, 25549918, 25549919, 238042, 238043, 7248900, 7248901, 16632494,
		16632495, 37184834, 37184835, 3934136, 3934137, 31120362, 31120363, 36454734, 36454735,
		14059218, 14059219, 9502912, 9502913, 24810294, 24810295, 47833150, 47833151, 63459724,
		63459725, 21830544, 21830545, 35083782, 35083783, 36750118, 36750119, 60695996, 60695997,
		15304996, 15304997, 29389880, 29389881, 43062130, 43062131, 37586164, 37586165, 4303694,
		4303695, 32719922, 32719923, 30133816, 30133817, 19691770, 19691771, 12694514, 12694515,
		36915336, 36915337, 15774426, 15774427, 61837002, 61837003, 3186138, 3186139, 16297838,
		16297839, 31232738, 31232739, 51663568, 51663569, 21282034, 21282035, 11616704, 11616705,
		18376636, 18376637, 291772, 291773, 54304530, 54304531, 1054106, 1054107, 35986490,
		35986491, 12614944, 12614945, 41286800, 41286801, 20624658, 20624659, 62433918, 62433919,
		39708662, 39708663, 33747208, 33747209, 9110260, 9110261, 11777868, 11777869, 31474018,
		31474019, 38573944, 38573945, 7006, 7007, 34120876, 34120877, 46961334, 46961335, 44816784,
		44816785,
	];

	const DAG_NODES_11550000: [[u8; 64]; 128] = [
        hex!("4f8a5188a0446df88386b3b5e16b7e5e70481c7fabc2925c86cc9cc892ce759194711edb6c55edbc6f6f4e7de943f8bce38d3ba50b88f7ac36f78ab178d373ea"),
        hex!("a565ffd7e51108117e75f10c5171afced5ac1510970118bac01ddceec353b48daaf6ea4f4b32081280224e6b3d0a6f3429c08358f282687b77f5dc9c2acf5d58"),
        hex!("16b99085654e95e2a8e740731f1ca1a076ed8de287134285aca1f2f454bc368f969144714a9f231b5a4cf9f6908c39daeebe2cf78d5e0a9cbe0d2f7901db4428"),
        hex!("4cbfb405c484cf47227d0e00922f59586282202d14ca87a93bccae6fc43b79425409c55a65f24599e8db8f7be332b139c321239323a214972e2b61fdadbadcc4"),
        hex!("4775a285cb153b9eb13b04dcc869721586c6419ad63aa8d6a2e448657fa106b12aaa42b18a19f84f5599a8305c966792669935c8f618bb7e0c8ab936c25d39f1"),
        hex!("021398b84c4680ce771c7e1e2a4745c08793b8a137caeadcc7cf40260ca042dd0ab9fe14c5d222dc7f0b212ed0224297de309431eaf1a7e3f95dd8854200444a"),
        hex!("413af2332e720185de6f5ebccdbe662580bca42d28011173cc83933a924e13d2d4e20f5490d2c6b18eac73fae54395befa19cf43cd4b98fe69cec930c08d7f32"),
        hex!("237617d366fdddffcf53894d41d7e5f17ad5cfa0eaa268329f630e820a89fc19f60d6114f33fe3ab58b23737a0653bff7b5eec2b00ec9c055f2267c91978033e"),
        hex!("08ae7a996bf54db330a67030d9b807f3a677be3c08ea8d4a179fbcf8e960d206a46428241cae4edc3f5e05355444a9736df442c326c9b4d568dbd5d37cbda4b1"),
        hex!("9bfc888ba04d76484bd9453235eca817d60b4d3b4723f42767122dbbbd49feb0eda7662da6801f56ff377251ba640741d15016673112ac21810836600560e173"),
        hex!("a8c6ca6313f58675ad48dbca8a1db26112696191dff2a54159d182cdf28d68b6008bf98af9b6f95b2daf4234456c5abb6080f1fa45f4afd78a26790b9b6a55ff"),
        hex!("d1144bb84174bf9bece6fd212b0f3556487b61caf4aaa4ce56102c2cec665981962059cc936494825dea97fd7aa69b82854a2cddd6953f26be96389a9be9cbd9"),
        hex!("eeba1cec0d56abcab386f6956bc432c4b4415e1fd1dbebbfa682b7d9c68752f2202f3039359928c523e80d8ee55efe41e3f1f2d7fe440274e621175e8932b46c"),
        hex!("2115f5ce4dbb903bfe9aca49af6f8324cd1491b84ed2b4ddb880011608a81348812d7a2e8ace8079de3d1ba800bc7a28f75215562e0267da4d8702ccf7ad5a7d"),
        hex!("c7c2a9414fb637805123abaecf9a557ea611a0a030eb628e20d1e1327b647e8e26f563934d5051e489551ec34c20ccf2404d98617e59f7294e7ebe408cf3cd17"),
        hex!("df1b75fc00f0968d48053c0f26b7f417af9c0e80f545bf274e2f675d6015c8a6b9f09f23714e5b0867fd48cd3637954570f0a0c1c4c112462e24f811451ff629"),
        hex!("dce533e3ccd6fa531e55f96a96bc58280560e7b12cd04c55278686b70592a1f66a4f85f1950f73f233ca0e7daa64117e2de79d77ce0cde6dcea4b594a7708df9"),
        hex!("c40e6a9fff86a78c6129c409fd0b031c7c33960f87685a22209a8cf0b7b7f457dbc9c1f69fcc0e66c23b6f830cb753c130fd511f460a091df7fc749f32588a25"),
        hex!("19eaffcd4755cd798080e1626574b2212fc16da8e0c9bc1f945e5dd9b22f02fdd4d2c21a94e37acfcebe5b8d472ea34ea02584ebf6b66b1bdb0f38eb8b039f9e"),
        hex!("4ad327b5a44b3b498461b9bf83097727d389b5903a1ec81404dd83f07887e887a2886d75eb6394ddbf35fcc5895b1607fb878b50e605c6636758b3fd5f524aa7"),
        hex!("c3ed093f4c7510a1c5ba54eb4e5c3e66706b74224de23db1e929e59e2dc6c119c2f1fb1458856857ceadc2c783461a34e68dd1b373487962aee9f6f88c712bb6"),
        hex!("453f432d8672811690597d3652b8f9fb795166eade81687108ba772347dadbc3e96536c728de4a0b8508e91cc086146576a3a01c83439f5aa1262d38c425795e"),
        hex!("da1373affa57d9493c115fe9a56ac18217eb3040db2cf0604bf0e071fe246bf2ce4c78f0a058c414cb9fdf3fec7ad2f5509f1b701ed591d80bd7ea3004d14e4b"),
        hex!("e73c4af58f8dbe699599157de9b1bf3d81af506c4e09747dfa329a38d45aa04f137054e637fe8537443d9ffb90a973131314ed4788523f8935aa64b5e1772776"),
        hex!("daea4e99afa32d5b5cdf9f629dbeaff25ea252cb065bedc6579ea610903f8c65aed0349fb065488832a1ef65ef40461882aed4c5dd999bb9ba84601739711334"),
        hex!("165bd986e951c9e2d809378c106e9e062a8e61f2b036f57a10ea7d3aeda009db4abab4052889c28fe4902ab00d4f401d867941ff973d2712afc240daad8f0a48"),
        hex!("491795291036d5af166c0ac7d6cdd8d8e35855d84f60358d477ca946de40bbcc59f58f06a93a44dfc77a068826d2f6068c24005f39b153ffcc9bad065b04ca83"),
        hex!("d508b1153caa672b50dee140beea5e6b540d326ddcd763cb33b5a5958f8fda06f15c73981b638b178da810f93143a4fdbddb72cdf9947d46c45e720d8e32abca"),
        hex!("7ebe7dca72b58394f9e29a3c45ce232abe670b4fd4707111fec54d149c7ddc3b00f28f9c6f007b783e9b05e1dd7640266c77fd9aab43b7184bc0e41f641c343f"),
        hex!("7e565bf90a6f58ad7fffde070f8e0c43b6deba1b1a5b6519eb2e20122f48efea65bca264bfafec09d9bb8937c7679b9dc9c82c66aa257e6cd974a3112f7fb79e"),
        hex!("319b616aab2872abd63620577a408b1f41c2a32a5fa1878fa6f886262b6e0e59f4c0ba85975a0a814a262d310723b269c7511921429e60516e20f0c9ebd2682d"),
        hex!("743ee1eac8acbb06b93631816cbb18e6188e0203ec2d026017ef9d6680bd6396ac5b3a900bdf7b98be17412d17cfcf89ec9595a723e827a018ffe086209492fb"),
        hex!("4edfdc522071ceab655021caef43cdeaee8f46111bd48eb95fab799705f8cef63e8e5fb264cdab81ad2183cf46ba0b5247458527c1334913af86fb0643e004ad"),
        hex!("d0c34b3bc1cff8b1a61de178df32c1de4f4d3494170661c680300fc131c45e60cae643ea20ee5082dde385dc0c96d8b0c23ca3c86c788eb203b98821b03622ea"),
        hex!("60e56e5d76bb3b8971e7ae7f05932feb30a4160b32262f852d8ff1e876846695857dfe3c47b78c67e30a63766f6e83ae272dc15c66bfe26022bd39a330501fb1"),
        hex!("bf63165f573e8f059744c7849172588101998255cc0f6c4a69e28c07812949a2c3a4837573a6320519812470987f7c9e55e6b105b1c8be440001063d24888a2e"),
        hex!("92dc400c4bb4222e1f61b3ca0d05c5c45ca0553c65c388d244701d49789694783e36c1da9bad20f4092093c3b89cad591919cd6168a0eeb429d32c15bb329b32"),
        hex!("ca5601980826a54c38471a12ac5212f720af402ec4bddacd7038b222534116cf2220583f232dba7ad15b4432c8f48a515d9912b3a040e3db1d00dd6daad91399"),
        hex!("2f8cda48d975b85baf15deac75e5c0fe46c87f88409f1edc3ed8c06843d872f240f9f8cb751d9437965c35b913b18dd1c397bcd6282aa30e722d7b90049414b7"),
        hex!("8118fcb3e92e2edfe216ff6d185e83fc441c05196ec2fc87bc102e33944b355b2e150ef35de00c2e59a69e5ad2b8048a260b5b2b2ad1cf6e314fb7ccba72321d"),
        hex!("eef575e9d3a0103554f109e044d3635eb97b7e4596a0a40273c9029386d24bc8e431aa4326cc6d2c96563cf3cae2c5a8442d17c0604ded8628ea1c91847f8161"),
        hex!("f31eda1d8628d3f152e04fe9f04885df99967f176f592a808d0cecb6b2b1e6fd4485edc373358130ebc61ab7bb9b15db55c107a58dbb4dfcd0e8c1bbb9333102"),
        hex!("1e06c4d45b3ff260b7a71feb109c20b781cea7a1cd7b1ac9eeac7268cd13ad88e082bd74163cf2fe96e3cf0ef271f31ffebdbee0272ad15a236cac6a84e8418f"),
        hex!("252626eddec47bb430727ec821c2c6a6e8729083d699ab727fda6d5abb670b60636a103e3e5da13ce99d13faab01dc1cb9533842f07b4ea8e2e87a2cfba86614"),
        hex!("386463a6eb305f36549d6ecf78f0e443bd356adff499c1f517c60365865c41fcec6e04144c79fdb3abc61e5ab2947c8b71a3d0891e0f35fa9466bc1bbd38bfe0"),
        hex!("2cfc6cb54cc3eedf50fd1060d5b230318418ea0a4367bd518960c3f046651f98f4f10e5cd12ba9815b5f5988a46e6532b1d914fb904e777395bcb6ab468b05e7"),
        hex!("4cbed9ff4e8675edd42c7704ea134a50351996b9506e5c9b024540a84afeee3d19fcd8d1ab88b157972c7d3361662795f3add1d180577cf411ae99305be620a7"),
        hex!("7aaaf7e41b5dfc84598a88bbf8e6a8e6cdd25805036d66ed0a2707e9a0d1a555d2ed3cd39644d5bfb8e62329eefedef4367ef82100db01f7aee0e5a78eab0c2f"),
        hex!("34ce60daf66708cb18c21f4505118540a72770f0c829d057f170555ff90c458cac987e88424091a8269ded70790112ead74f8829a03fa3ef9007f20f7fcd4849"),
        hex!("e7cf70cbc785c6194efb416c4b7929cd84a7ab5a1c09f269c14d5ca5c77cbe4db040ebe39b2d45b988720b610435a52afcda696e41283306598f396e7aec4d36"),
        hex!("8d085239e6c6c5c03216cda23de434db60f6ccf490dcbf3d48fe3272a266dc739425ac45229cdcfabf1ca65f2e667ae4e0a2f4df85a8d2277ece83ab5ada7333"),
        hex!("6fd0e981e291a419091e82805562940c5283fb87dbdc31670b6e68b89581cf629ed19d03ed8d7e740c00a0ef0cfe6b791c6b56ce3da494d2bda65ef5abac881f"),
        hex!("1214beef209f07f99fc08abe7d6ffd8461dae9e79dfbf795534626d55b771988b356c7b51000174cd0cef59c0cd2fdba53fb5f358cad3114acb2356cac3d2898"),
        hex!("69164c77bef4324a7ada95062c9565f05cf15ef371169e1457fca2a40fede7ec5404678e1edcdb60ac5594c1103c98ca0042fa9f014eaa00cc43dc0dfd1b2dd4"),
        hex!("ce32ea648f5c4664057b4a89a69f2a4549f78fb93b64075068607223ca72e9a89a123db7ca3e988419ba7fb5febe91fb9ee640102f8a13f546f51831b2cc17f2"),
        hex!("6b7291fb2b958bda392512f149512b1c0362d6c41bd8c9fda08746994f63f4c18b25607ac57ee94e9650bca895acb472a806c21f7e8b397ae7eeda687332ed49"),
        hex!("a675b0a790b755b8b2b0d74c3f5db30d367d34e632ca7c7c6e86b238728f1e5c8963222eb7d9f54f07e8858a76c89c53e8b8b191f5fdae464857a685356c19d2"),
        hex!("a6a167cce23347f8da947f742f568f46b0cc19105d76b0946d25e03eb8424e42d674254167f0f10c69f3acc57b69413b66454be97ecbcc5afcc99da9270d5122"),
        hex!("a787e20495678ff66fe19ac8f9bef0daafd35c3c7cb55891673dd6db53e214dc11cabe06924a861bc6d0f375ff7840d891d78233c04232c7c8468bf0d7a6f85c"),
        hex!("e4834d78c5cd070e72ebacc9bc02ca96513d1ca31556bacb3d244fb08a061ddd121e7d86251d4aa03ae70da2a7eec574fe4c580f3fc30dd2fb3278e3ab9223e7"),
        hex!("13aef40690769ce954187eca2109359ea04f14040f798bf5b0037070d35ad048ff96e96c0184b900c45442e0b256cb3d43f069a208304408762052d693ff8e09"),
        hex!("3a22b3ff2ed34708ecc36077ecc65ed0b3eee6aca57e35b341984656e5d30d9337e8a6aa24e53d80320609cd4e1283a1df5ff9b71041cae2a3f8c85ea5a65e1e"),
        hex!("d7ced71ed6091437eb90b2469757aa8fb0c50a512f8db4cbee9a2f5fc4ab1b2872b71c0bb0107b960b06b7b53e9be0b30e02bdec562ec583e30214d7fd760c9f"),
        hex!("60d82f3f12934e1749bdf8996937d35095b3f1bdb4f4e0263d8af1235ca222321a87cac2bc22b13f2083cecd533a8c3ca697d8a9852da81932aaf5418e0215b2"),
        hex!("9d4ecc3e989a0c53ff8a03c0fae9d4523c67f8be6222d8df610af8ca79af20ef6d952b8e1dd7de0891f3bff7674b24ab29b3f219f7c278b534d65d3dfed9b494"),
        hex!("f2ffde1fed2cd43c1ca516d04a88a6d7d96581d40a776b09ade95f0eb6f7b0e1a10cf4c6177414889d7fee91bb308ac816de6c343741add27e5535ee8de4d834"),
        hex!("5ac53b1e3b33220d23c02f7d03ce8efb9d6f53d4eb8169287e9844b849f95bf5e3a6a341abb59641e3f38633987063469e2456e045185eb08c12a95d32660e64"),
        hex!("b88a40373ff717018096ae985833d08fadf7afb8b551f61e0d7289679294c7f594dd19dc5a292c33c6b10c526f6bb056f5e2575140e69191cc8e6dd6004bbdc9"),
        hex!("7a378dc1b7e9634a04fea8a06aba820b703398909466429e5e2a908fed65bf3d08f5cf269bca2b3a56cf4b141d9cf7f3a7ce7d06e5d9c5150888659c3456023a"),
        hex!("fa669c4c379b7c9696856e5ae3dee50f3662c7330bcf5a0f2a25ed1f4225d1d196a64b6bc87b50812c2ec6a7dedd9d5b76691c6e58eab8927ede095e011b76d4"),
        hex!("5827e430e34571541be3ee8fae0a756e1ce3e6ba53bd5302bbb7507f4c57442342d535b8a0e2187b43648a96ed460ffbb21de4def12c5eac444921f9b75337b2"),
        hex!("d7d2a92b9db18f37ba37344de0be8df6a6a9d34a73cd0b44754fa09533f3db22bb83aab8dea6fd5063996222581390400cb222cf9e7ccde19e9c41767439bec7"),
        hex!("fd1abbf4912821987fcfe5f94714ff0e6c8393560e5b37840fde21343e419796fa419a79fef0b44c226ce9952551bb30f2f7b7ee6aa6bea208083471b148d76d"),
        hex!("75eb3b45e48ebc00300beddc8f72c958aa45bf3de59502d0ab271643c11e46611b14d20f4df4cc0fd9dc1e7eb36703ac8814fbe753dcffc6b08c97fc47ebedae"),
        hex!("fcec1909037a43ba46df62d21be622afb256cbe3d671ea7003da8f7241e205932bb2c2f4292ad97f68a627d2b8b2ff7767e9e3bfc093b1340ecebb9942b03fc7"),
        hex!("3fc7a7621ab8abbd900dcea77f7a51b3af549167fdc4b8a362fd5c822943a2105af3b29b9c46042659233fd1c7b968ad93007929c7e56f10a5a60a400a755957"),
        hex!("2b0c5c1952fc863c0f1b176d75a702ed3e45e8e36faa2364b6df918c4309cb05c2737f6ee5a7429e387dba6254b3924c762df4f539dbe9b017967fb2400e12ef"),
        hex!("1c1e07459d7411dffed20fca05c70d8f33e363e317706d9f0a59a4403056fdc3b90dd77641ccd23d7d868c4f10764ca1b062b645409b367162637ce77571db91"),
        hex!("7556d6157da1db9768ae501a4ddc93cf9bfc9933beecaae5bf0a904dd2fa954cf032d08660aa50591af4de6b5d7ebc78870b6fef636cc1801d6530d28e6783fe"),
        hex!("01222084d2e4a8dc10e9c991f7d07aae839915ea8e00f5b26ae4496eabcc0ffa1b59c7fa4dc09ea5d59543c2f8f3d5d9ccf4b2fc07c7150a64ccf53cf34c162d"),
        hex!("957969a51fd22a11000bbd10baf5f4f12e4066778394e3c4e17aebfe0f50d407f7f3f2c4bc612303c8de7aa579be78320bf3bbb80d043bca2ef246981bcf621f"),
        hex!("933ea0517e02ecfc2e27df9d9dbaa57614144ea6cbeab7a4965308b7954d5a6c2419a518ab309cf57f6917cf40b1475c2bf749cbddbf9c16ce8a75bc1e4c2d0a"),
        hex!("72644f2b70f5a1354ba05fa652339a838101af55ebf006ec74c385fe1420a3115a06f263ba6c0e47e8d7bcb33694997acd83c75155ecfbf410597de031e35d8a"),
        hex!("9e16e652710d4fe93c328a606089af6087b12feb3ecd5a6ac09c8dfd369d7136c8acb9a26f92f38cda4c0df1cf9b266cac90679478de1784aa4948d76c7efb75"),
        hex!("845245d7c4c8768df37242908397c98129825e1b1ea94895f6cd96eae6dae76892181e9c71821768a6c9e6170dd2f49d410efdbe784366702316c7c7911cc442"),
        hex!("86c367500d7c9fdfafd381a8a84c9af83db0c5b588d5f225bd423a1d446c923a8f66e5603eccaf95f4e56ca95830ac30da5cc2ac4722b9dccf9a0a162eaaef95"),
        hex!("ef2203464f0eff26c7fb98e4f140a04bb80a223c9ee611059bf604f2e68a676006775b4fe6208ca3a4479730f08dd0da3fb0aea2d7f6271b632dc0fc8a598ee9"),
        hex!("e360bb958304e18c89ebc7b3cef443579f142d71bd1bf3fa1f873969542ca7fc7464ee636889e4da1235a8b1db22a105b78eee9f7de4276be3740c421294e530"),
        hex!("acad1568dcce42444d2654ab45571f6a9b111f072424bf18eb3de0eb806bc3031455c5249f9d011db1b6e0a7f82051513557aa305c502a082c2b82aa27432673"),
        hex!("a3d3773de64b72fe8d8e24d8be4498fb9e380f2ff6def5eb09064885370e05dd6267ccb7224ea18d1eda473bfe7def1a5a8d08c8667cf8cc22856ec78f793614"),
        hex!("d2f3fd5a9c23d9dd5640b8b5017bf6905bc661de0d5e6dafb1e5f8a80fe0d2599a8a4fd5d50e9a631027821d36e8f357100336e3dab4b93e338bf1dbbefe43e2"),
        hex!("ca44e67d605081401935e034e98cbf4053c25c113bb54df7f9c85e296059c1cedfb9432be68c9dc87aa7f1106d83df1ad5a04419d960395de07d37d1bb671074"),
        hex!("821b7d8a4546c4cf41f57b93988dc47f52805af933a4bea1ec5a7e5712fb37fcce874e8f8b89d708ec8d28b3aa29ec16892ef5af64bc4093e825a021102fef47"),
        hex!("5ad310934d247d90167f027ca8cf0c2cce6dce8ab6d4ed823134b927b8ffbcf8fe90c345612e9c63940d29d38e48ba7253ed91aa5ca40c5e62d94e5322cea745"),
        hex!("49154090f51da0f58cf14a1b3636a47ab2c769c686dec5d900c0f1eab134d3a89aaf6b59156815d0b022a83163a835e01fb86666229123bf80207d065b125cc6"),
        hex!("fc126f965cde55067e2cc955e5a73325695523c238b41a8bc63c0915812c68819f4bee8d8c5ceba22fb759896eb3fb3eec475a3c201bf946af31965b9e5e54e4"),
        hex!("b13f70703389f688a3db8a98759d3138ed586a30b76c7418abe066a91828dd1626e8a0c281e0b1cff30cc2ca4e5902dec1a85d2141892090d40737e6b24388c8"),
        hex!("4a474f8324e161287b3771e6f759af3c919c1b2a47102424b078b183bd1124144df07c04aeeb341e53f9944d06e9319998d420147703b0c813e83fd2d9d73b53"),
        hex!("e7b5f28991d68b0cf1d03d7b794546c56bc7c452ebb63b241910f08a3cf22cd1783952aac7303c44d12af876bb0f3edae8907e76ab12360126149b706820ac24"),
        hex!("9495582f9a3643fbece23fc39b1636e225473bc6cb063ca9aedd4be87fdd28eccd794cf02d2661f6594dba78864c4cf3ba1e59af0ba6f4263fb011a90d808fd8"),
        hex!("4d8f7579483bfa21b5ebc1ce5c0ab4351b13f1e3a887fd379539ee2f7825ddb6bc2eb11723df69b069d51f2e1f4b2dd9a9057337f39f6dcac511bcc259589aa4"),
        hex!("225f2eec709f4f8d4be4eb303dfc4bc40286b32f2de2fdffa2be2da6f48f30d5651c8296e15aacca898826ece0bf9822872c5b54775c2fcd6760f9a55889ca64"),
        hex!("588ac0f3b49d9929fd4589a71c98b2abee357b639e1da14e260a8030ec28615461d1672ae921dbde19f91ba61f20ea6047e578a5448db2dbd368dc5c825ec2bc"),
        hex!("a9e0be1848dd1142e24aa56b5c50fed3b684c1884d8f3f9cdf77492df1575436be468d305390dcafe03813fac4b67e9b60be09d68e9939f16c569e40681a58df"),
        hex!("c04bc11cdea62f46b66051d0ccde6d02631f7087b3f52ba074576bd9aea51adf0b27b784155e649dc7f54fa813887c5cb2c988c9d38738ff7245c9a3c0b20aeb"),
        hex!("c7faf0bdfd4c8d2d2b4774d911fbeaee5bafbef3aad89946900a5cd99eb73fc3667c207c120ee749fd38fa4c453f9e8aea104f832e75f6ceb647a9efb17ecb70"),
        hex!("9bd24f855de27f045b05c7f54ed308be75d7e778679ab2bd0f7350b8a5a98a7e4ea15da398914b845e62900c007b7cfbf8d40beddf80a5715417d52659bce081"),
        hex!("4e47e6cead640eea245d3c061673542e63bafd9edaefacecb345c1177909da0d44713eec6afa5f36c85dcc48481bbf485801efa111e7723a29260bbc2c112a31"),
        hex!("11a7cc5bde8e296a525ab14fd19e98d347fdaf3626813f1279d8f496295eda0ee4ea427a6e7dd2d485def8a98ca34036f773e09d69aef8abf6f397596be66f40"),
        hex!("eb2074948b85204e74119b925179f769971d8868de055a11c3bbee3fc4590b870a3745e81ba63f75f00128d3839dce2c8ca5ce567711a9a02223a9e445b49b56"),
        hex!("fa3a9787629af257625e8079d51d6f94e5daa80b253549da3f4e198e805fe197cad179010b94ffef5c93c005c89b4f81e6a60af0211967dfb9a32bc5aacfb08c"),
        hex!("d4734e04f5c3a1c875658f2f1c768b0b95023d944f6aca720a75c039d0c958c0d74132cbc034cda19388e1de9c9b3eac6cad6175db10322d94500e4b03cf64d9"),
        hex!("e83a636418569947f3395f105fa79b71b4d62fbb3b8e04858eac70f0ff3d65d4f8b3209b16faadde6ec6f3b18c701b6db78b36bab651071adcdffdf20d6ccbe7"),
        hex!("15426518b6b2ede6cd1be89b064f1ac239329e522d85d0412c7dfd5dd1f8127add328bfa41e6fe26acc46de63016491fd47f5768380c9cb5577bd589690f6912"),
        hex!("84faefe41e8a1bd49ddfe9fdb95a5fe4e0d5e71f4f1c0016af599cd482d0b509aa67931c02f9dac7221be05dc87e0f03a5bb11945b732442df6697bee788eecf"),
        hex!("bbcd516d74d3510f54c0861476f2293af5999982620be8a8544a201ed308f4e48005875982cf9d2341cab53ff3a9b1dcc16789ea3ebca54801ac5c2f750b10e6"),
        hex!("6fb9180a6579f54ee68569f983834ad6d15ea29227c1c54cb5d3fcb97081665c11d82def47eaad5b0467804236722b5167bd9ed2a336212b9a1622cff2aa27e8"),
        hex!("6e79a3f3f6084c1063020f2270ece972232cf22b3a6a8c88079cc1cca9f2d023cf14398e59dee93211c637fcb85d01422f36163b72395a40528261a00a1bc12f"),
        hex!("46cf6ce2941c2e467f2149b77e5216c1730ee1c3d4ae62476dcf7c3cfc095f221439d4f2df25c23d273dcf3b50bd24ffbc902266ffd5c29125914c17477dcdcc"),
        hex!("339236459ff10a7902501de1079a1822eeda9f1d46544dbac8c182c15158b4ab7be70a036aaa2e507441f2a526b9191e1d3975f55bc98eea94746b5492f6840c"),
        hex!("32b62e97bc132a71e096de6b9b6ce8f5d718387ecce0799e54c8be9e8ecd00400c53d99e796768742c619023a52ff2967c84e7509339f4504354a6c8dfe9e362"),
        hex!("9454b1d497192c6c67b94f0216749f447fc4885768b75394cd8bd4fe8f9402f8d1983362cdf7fa05c7c2bfc9c0c69333b2b5dcbfda0ce642ee58231ac3dad154"),
        hex!("10520f013c87948b17c1083b1a59ef70a81a17d84985235fe478dd1d3607a3fd38dca172107c08bb50b9d9cab56c28d5f9a33c10c2b79a11af481d6b5396e3f1"),
        hex!("7d2d4c1062dedd873fccd80357f115fba3d817e74ba6344704b2cf211efefc1d7fa4c1e9a5c1d96a7ba87ad5e0eb61327c0d199cf98fcbc6babb6192da7af1e3"),
        hex!("ac65c4d2e0662b09361e6f53c1b494c6c5537d264ee57f4a12430396ce73d1d8128f4b3eb2ea5c3b216965dc30bc2c76064e088cca72f587a05800d811a26450"),
        hex!("be49ec5003421b6d9d842747d05842992282fef8110b5cf19634bdefad7683d86cf1edf4cfa9ae2b67c13376b10aac3d3c157efe57ea15ddc122f778f7a02ee0"),
        hex!("e90182b99ea1095f42f2fc1a7ad15a62f434a2ef66f0fcfa11563e88013cd93d50823e1a522d48bcf85bde8077ec964163f3790208b13054e6891b5c166874a8"),
        hex!("0e9df2c6eb40ee8a86be7b7bc8250a3929373364478017d79f8e57f4fc8d49de1e45c943f2fc745d8d383044b21a73e04713d09649060a8995e2456b6d3523c6"),
    ];
}
