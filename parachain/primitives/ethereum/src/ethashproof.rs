use ethereum_types::{H64, H256};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

/// Blocks per epoch
const EPOCH_LENGTH: u64 = 30000;

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

pub struct EthashProver {
    /// A LRU cache of DAG caches
    dags_cache: EthashCache,
}

impl EthashProver {
    pub fn with_hashimoto_light(max_cache_entries: usize) -> EthashProver {
        let dags_cache = EthashCache::new(max_cache_entries);
        EthashProver {
            dags_cache,
        }
    }

    pub fn hashimoto_light(
        &mut self,
        header_hash: H256,
        nonce: H64,
        header_number: u64,
    ) -> (H256, H256) {
        let epoch = header_number / EPOCH_LENGTH;
        let cache = self.dags_cache.get(epoch, header_number);
        let full_size = ethash::get_full_size(epoch as usize);
        return ethash::hashimoto_light(header_hash, nonce, full_size, cache.as_slice());
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use hex_literal::hex;
    use rand::Rng;

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

    #[test]
    fn hashimoto_light_is_correct() {
        // https://etherscan.io/block/11090290
        let header_partial_hash: H256 = hex!("932c22685fd0fb6a1b5f6b70d2ebf4bfd9f3b4f15eb706450a9b050ec0f151c9").into();
        let header_number: u64 = 11090290;
        let header_nonce: H64 = hex!("6935bbe7b63c4f8e").into();
        let header_mix_hash: H256 = hex!("be3adfb0087be62b28b716e2cdf3c79329df5caa04c9eee035d35b5d52102815").into();

        let mut prover = EthashProver::with_hashimoto_light(1);
        let (mix_hash, _) = prover.hashimoto_light(header_partial_hash, header_nonce, header_number);
        assert_eq!(mix_hash, header_mix_hash);
    }
}