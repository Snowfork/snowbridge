use codec::{Encode, Decode};
use ethereum_types::{H64, H128, H256, H512};
use sp_io::hashing::{keccak_256, keccak_512, sha2_256};
use sp_runtime::RuntimeDebug;
use sp_std::cell::RefCell;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

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

#[derive(Default, Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct DoubleNodeWithMerkleProof {
    pub dag_nodes: [H512; 2],
    pub proof: Vec<H128>,
}

impl DoubleNodeWithMerkleProof {
    pub fn from_values(dag_nodes: [H512; 2], proof: Vec<H128>) -> Self {
        Self {
            dag_nodes: dag_nodes,
            proof: proof,
        }
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
            let (ref mut t, _e) = self.recently_accessed_epochs
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
        Self {
            dags_cache: None,
        }
    }

    pub fn with_hashimoto_light(max_cache_entries: usize) -> Self {
        Self {
            dags_cache: Some(EthashCache::new(max_cache_entries)),
        }
    }

    fn dag_merkle_root(&self, epoch: u64) -> Option<H128> {
        DAGS_MERKLE_ROOTS.get((epoch - DAGS_START_EPOCH) as usize)
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
            return Err(Error::UnexpectedNumberOfNodes);
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
        return ethash::hashimoto_light(header_hash, nonce, full_size, cache.as_slice());
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use artemis_testutils::BlockWithProofs;
    use hex_literal::hex;
    use rand::Rng;
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
        let header_partial_hash: H256 = hex!("932c22685fd0fb6a1b5f6b70d2ebf4bfd9f3b4f15eb706450a9b050ec0f151c9").into();
        let header_number: u64 = 11090290;
        let header_nonce: H64 = hex!("6935bbe7b63c4f8e").into();
        let header_mix_hash: H256 = hex!("be3adfb0087be62b28b716e2cdf3c79329df5caa04c9eee035d35b5d52102815").into();

        let mut prover = EthashProver::with_hashimoto_light(1);
        let (mix_hash, _) = prover.hashimoto_light(header_partial_hash, header_nonce, header_number);
        assert_eq!(mix_hash, header_mix_hash);
    }

    #[test]
    fn hashimoto_merkle_is_correct_block_3() {
        // https://etherscan.io/block/3
        let block_with_proofs = BlockWithProofs::from_file(&fixture_path("3.json"));
        let header_partial_hash: H256 = hex!("481f55e00fd23652cb45ffba86a08b8d497f3b18cc2c0f14cbeb178b4c386e10").into();
        let header_number: u64 = 3;
        let header_nonce: H64 = hex!("2e9344e0cbde83ce").into();
        let header_mix_hash: H256 = hex!("65e12eec23fe6555e6bcdb47aa25269ae106e5f16b54e1e92dcee25e1c8ad037").into();

        let (mix_hash, _) = EthashProver::new().hashimoto_merkle(
            header_partial_hash,
            header_nonce,
            header_number,
            &(block_with_proofs.to_double_node_with_merkle_proof_vec(DoubleNodeWithMerkleProof::from_values)),
        ).unwrap();
        assert_eq!(header_mix_hash, mix_hash);
    }

    #[test]
    fn hashimoto_merkle_is_correct_block_11090290() {
        // https://etherscan.io/block/11090290
        let block_with_proofs = BlockWithProofs::from_file(&fixture_path("11090290.json"));
        let header_partial_hash: H256 = hex!("932c22685fd0fb6a1b5f6b70d2ebf4bfd9f3b4f15eb706450a9b050ec0f151c9").into();
        let header_number: u64 = 11090290;
        let header_nonce: H64 = hex!("6935bbe7b63c4f8e").into();
        let header_mix_hash: H256 = hex!("be3adfb0087be62b28b716e2cdf3c79329df5caa04c9eee035d35b5d52102815").into();

        let (mix_hash, _) = EthashProver::new().hashimoto_merkle(
            header_partial_hash,
            header_nonce,
            header_number,
            &(block_with_proofs.to_double_node_with_merkle_proof_vec(DoubleNodeWithMerkleProof::from_values)),
        ).unwrap();
        assert_eq!(header_mix_hash, mix_hash);
    }

    #[test]
    fn hashimoto_merkle_returns_err_for_invalid_data() {
        let block_with_proofs = BlockWithProofs::from_file(&fixture_path("3.json"));
        let header_partial_hash: H256 = hex!("481f55e00fd23652cb45ffba86a08b8d497f3b18cc2c0f14cbeb178b4c386e10").into();
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
            prover.hashimoto_merkle(header_partial_hash, header_nonce, header_number, Default::default()),
            Err(Error::UnexpectedNumberOfNodes),
        );
        proofs[0].proof[0] = H128::zero();
        assert_eq!(
            prover.hashimoto_merkle(header_partial_hash, header_nonce, header_number, &proofs),
            Err(Error::InvalidMerkleProof),
        );
    }
}
