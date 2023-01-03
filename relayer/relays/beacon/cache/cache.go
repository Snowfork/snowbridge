package cache

import (
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

type BeaconCache struct {
	LastSyncedSyncCommitteePeriod uint64
	Finalized                     state.FinalizedHeader
	mu                            sync.Mutex
}

func New() *BeaconCache {
	return &BeaconCache{
		LastSyncedSyncCommitteePeriod: 0,
	}
}

func (b *BeaconCache) SetLastSyncedSyncCommitteePeriod(period uint64) {
	b.mu.Lock() // mutex lock since both the finalized and latest headers write to this slice in separate Goroutines
	defer b.mu.Unlock()
	if period > b.LastSyncedSyncCommitteePeriod {
		b.LastSyncedSyncCommitteePeriod = period
	}
}

func (b *BeaconCache) LastFinalizedHeader() common.Hash {
	return b.Finalized.Headers[len(b.Finalized.Headers)-1]
}
