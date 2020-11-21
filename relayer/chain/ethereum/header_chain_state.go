// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"fmt"
	"sync"

	"github.com/tranvictor/ethashproof"
	"golang.org/x/sync/errgroup"
)

type EthashproofCacheState struct {
	sync.Mutex
	currentCache *ethashproof.DatasetMerkleTreeCache
	nextCache    *ethashproof.DatasetMerkleTreeCache
}

// HeaderChainState is a helper for tracking where we are
// in the Ethereum chain. It warms up caches as we go.
type HeaderChainState struct {
	currentBlock          uint64
	newestBlock           uint64
	ethashproofCacheState *EthashproofCacheState
	eg                    *errgroup.Group
}

func NewHeaderChainState(eg *errgroup.Group, currentBlock uint64, newestBlock uint64) (*HeaderChainState, error) {
	cacheState := EthashproofCacheState{
		currentCache: nil,
		nextCache:    nil,
	}
	state := HeaderChainState{
		currentBlock:          currentBlock,
		newestBlock:           newestBlock,
		ethashproofCacheState: &cacheState,
		eg:                    eg,
	}

	// Block until cache for current epoch is prepared
	cache, err := makeEthashproofCache(currentBlock / 30000)
	if err != nil {
		return nil, err
	}
	cacheState.currentCache = cache
	// Asynchronously prepare next epoch's cache
	eg.Go(func() error {
		return state.prepareNextEthashproofCache()
	})

	return &state, nil
}

// GetEthashProofCache returns the cache used for proof generation. It will return
// immediately if `number` is in the current or next epoch. Outside that range, it
// will potentially block for multiple minutes to generate the cache. Calling
// GetEthashproofCache will also update the current epoch to `number` / 30000.
func (s *HeaderChainState) GetEthashproofCache(number uint64) (*ethashproof.DatasetMerkleTreeCache, error) {
	epoch := number / 30000
	cacheState := s.ethashproofCacheState
	if epoch == cacheState.currentCache.Epoch {
		return cacheState.currentCache, nil
	}

	// We're locking to avoid nextCache being changed concurrently in
	// prepareNextEthashproofCache.
	cacheState.Mutex.Lock()
	defer cacheState.Mutex.Unlock()
	if epoch == cacheState.currentCache.Epoch+1 {
		// Try to swap to the next epoch's cache without blocking
		if cacheState.nextCache != nil {
			cacheState.currentCache = cacheState.nextCache
		} else {
			// Retrieving the next cache failed previously. Our only option is to retry
			// and hope it was a transient issue
			cache, err := makeEthashproofCache(epoch)
			if err != nil {
				return nil, err
			}
			cacheState.currentCache = cache
		}
	} else {
		cache, err := makeEthashproofCache(epoch)
		if err != nil {
			return nil, err
		}
		cacheState.currentCache = cache
	}

	cacheState.nextCache = nil
	s.eg.Go(func() error {
		return s.prepareNextEthashproofCache()
	})

	return cacheState.currentCache, nil
}

func (s *HeaderChainState) prepareNextEthashproofCache() error {
	cacheState := s.ethashproofCacheState
	cacheState.Mutex.Lock()
	defer cacheState.Mutex.Unlock()

	// prepareNextEthashproofCache should only ever be called after
	// nextCache has been set to nil
	if cacheState.nextCache != nil {
		return fmt.Errorf("prepareNextEthashproofCache encountered non-nil nextCache")
	}

	cache, err := makeEthashproofCache(cacheState.currentCache.Epoch + 1)
	if err != nil {
		return err
	}
	cacheState.nextCache = cache
	return nil
}

func (s *HeaderChainState) RecordBlockSeen(number uint64) {
	s.newestBlock = number
}

func (s *HeaderChainState) RecordBlockForwarded(number uint64, maybeMoreBlocksAtNumber bool) {
	if maybeMoreBlocksAtNumber {
		s.currentBlock = number
	} else {
		s.currentBlock = number + 1
	}
}

func makeEthashproofCache(epoch uint64) (*ethashproof.DatasetMerkleTreeCache, error) {
	cache, err := ethashproof.LoadCache(int(epoch))
	if err != nil {
		// Cache probably doesn't exist - create it
		_, err := ethashproof.CalculateDatasetMerkleRoot(epoch, true)
		if err != nil {
			return nil, err
		}

		return ethashproof.LoadCache(int(epoch))
	}

	return cache, nil
}
