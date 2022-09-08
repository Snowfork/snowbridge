package cache

import (
	"sync"

	"github.com/ethereum/go-ethereum/common"
)

type BeaconCache struct {
	LastSyncedSyncCommitteePeriod uint64
	// Stores all successfully imported (in the parachain) finalized headers. 
	// Used to determine the execution headers that needs to be backfilled.
	FinalizedHeaders              []common.Hash
	// Stores the last attempted finalized header, whether the import succeeded or not.
	LastAttemptedFinalizedHeader  common.Hash
	HeadersMap                    map[common.Hash]uint64
	mu                            sync.Mutex
}

func New() *BeaconCache {
	return &BeaconCache{
		LastSyncedSyncCommitteePeriod: 0,
		FinalizedHeaders:              []common.Hash{},
		HeadersMap:                    map[common.Hash]uint64{},
	}
}

func (b *BeaconCache) SetLastSyncedSyncCommitteePeriod(period uint64) {
	b.mu.Lock() // mutux lock since both the finalized and latest headers write to this slice in separate Goroutines
	defer b.mu.Unlock()
	if period > b.LastSyncedSyncCommitteePeriod {
		b.LastSyncedSyncCommitteePeriod = period
	}
}

func (b *BeaconCache) LastFinalizedHeader() common.Hash {
	return b.FinalizedHeaders[len(b.FinalizedHeaders)-1]
}
