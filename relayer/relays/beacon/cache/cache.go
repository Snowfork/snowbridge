package cache

import (
	"sync"

	"github.com/ethereum/go-ethereum/common"
)

type FinalizedHeader struct {
	// Used to determine the execution headers that need to be backfilled.
	Headers []common.Hash
	// Stores the last attempted finalized header, whether the import succeeded or not.
	LastAttemptedSyncHash common.Hash
	// Stores the slot number of the above header
	LastAttemptedSyncSlot uint64
}

type BeaconCache struct {
	LastSyncedSyncCommitteePeriod uint64
	Finalized                     FinalizedHeader
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
