package syncer

import (
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
)

type BeaconCache struct {
	LastSyncedSyncCommitteePeriod uint64
	FinalizedHeaders              []common.Hash
	HeadersMap                    map[common.Hash]uint64
	mu                            sync.Mutex
}

func NewBeaconCache() *BeaconCache {
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
