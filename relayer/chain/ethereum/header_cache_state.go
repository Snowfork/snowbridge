// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"fmt"
	"sync"

	"github.com/sirupsen/logrus"
	"github.com/tranvictor/ethashproof"
	"golang.org/x/sync/errgroup"
)

type EthashproofCacheLoader interface {
	MakeCache(epoch uint64) (*ethashproof.DatasetMerkleTreeCache, error)
}

type DefaultCacheLoader struct{}

func (d *DefaultCacheLoader) MakeCache(epoch uint64) (*ethashproof.DatasetMerkleTreeCache, error) {
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

type EthashproofCacheState struct {
	sync.Mutex
	currentCache *ethashproof.DatasetMerkleTreeCache
	nextCache    *ethashproof.DatasetMerkleTreeCache
}

// HeaderChainState is a helper for tracking where we are
// in the Ethereum chain. It warms up caches as we go.
type HeaderCacheState struct {
	ethashproofCacheLoader EthashproofCacheLoader
	ethashproofCacheState  *EthashproofCacheState
	eg                     *errgroup.Group
	log                    *logrus.Entry
}

func NewHeaderCacheState(
	eg *errgroup.Group,
	initBlockHeight uint64,
	log *logrus.Entry,
	cl EthashproofCacheLoader,
) (*HeaderCacheState, error) {
	cacheState := EthashproofCacheState{
		currentCache: nil,
		nextCache:    nil,
	}

	cacheLoader := cl
	if cacheLoader == nil {
		cacheLoader = &DefaultCacheLoader{}
	}

	state := HeaderCacheState{
		ethashproofCacheLoader: cacheLoader,
		ethashproofCacheState:  &cacheState,
		eg:                     eg,
		log:                    log,
	}

	// Block until cache for current epoch is prepared
	cache, err := cacheLoader.MakeCache(initBlockHeight / 30000)
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
// might block for multiple minutes to generate the cache. Calling GetEthashproofCache
// will also update the current epoch to `number` / 30000.
func (s *HeaderCacheState) GetEthashproofCache(number uint64) (*ethashproof.DatasetMerkleTreeCache, error) {
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
			cache, err := s.ethashproofCacheLoader.MakeCache(epoch)
			if err != nil {
				return nil, err
			}
			cacheState.currentCache = cache
		}
	} else {
		cache, err := s.ethashproofCacheLoader.MakeCache(epoch)
		if err != nil {
			return nil, err
		}

		if epoch == cacheState.currentCache.Epoch-1 {
			cacheState.nextCache = cacheState.currentCache
			cacheState.currentCache = cache
			return cache, nil
		}

		cacheState.currentCache = cache
	}

	cacheState.nextCache = nil
	s.eg.Go(func() error {
		return s.prepareNextEthashproofCache()
	})

	return cacheState.currentCache, nil
}

func (s *HeaderCacheState) prepareNextEthashproofCache() error {
	cacheState := s.ethashproofCacheState
	cacheState.Mutex.Lock()
	defer cacheState.Mutex.Unlock()

	// prepareNextEthashproofCache should only ever be called after
	// nextCache has been set to nil
	if cacheState.nextCache != nil {
		return fmt.Errorf("prepareNextEthashproofCache encountered non-nil nextCache")
	}

	cache, err := s.ethashproofCacheLoader.MakeCache(cacheState.currentCache.Epoch + 1)
	if err != nil {
		return err
	}
	cacheState.nextCache = cache
	return nil
}
