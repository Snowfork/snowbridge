package syncer

import (
	"sync"

	"github.com/ethereum/go-ethereum/common"
)

type BeaconCache struct {
	SyncCommitteePeriodsSynced []uint64
	FinalizedHeaders           []common.Hash
	HeadersMap                 map[common.Hash]uint64
	mu                         sync.Mutex
}

func NewBeaconCache() *BeaconCache {
	return &BeaconCache{
		SyncCommitteePeriodsSynced: []uint64{},
		FinalizedHeaders:           []common.Hash{},
		HeadersMap:                 map[common.Hash]uint64{},
	}
}

func (b *BeaconCache) AddSyncCommitteePeriod(period uint64) {
	b.mu.Lock() // mutux lock since both the finalized and latest headers write to this slice in separate Goroutines
	defer b.mu.Unlock()
	b.SyncCommitteePeriodsSynced = append(b.SyncCommitteePeriodsSynced, period)
}

func (b *BeaconCache) LastFinalizedHeader() common.Hash {
	return b.FinalizedHeaders[len(b.FinalizedHeaders)-1]
}
