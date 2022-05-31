package syncer

import "sync"

type BeaconCache struct {
	SyncCommitteePeriodsSynced []uint64
	FinalizedHeaders           []uint64
	Headers                    []uint64
	mu                         sync.Mutex
}

func NewBeaconCache() *BeaconCache {
	return &BeaconCache{
		SyncCommitteePeriodsSynced: []uint64{},
		FinalizedHeaders:           []uint64{}, // TODO rather cache by block root, than slot. Need SSZ lib to do that.
		Headers:                    []uint64{}, // TODO rather cache by block root, than slot. Need SSZ lib to do that.
	}
}

func (b *BeaconCache) AddSyncCommitteePeriod(period uint64) {
	b.mu.Lock() // mutux lock since both the finalized and latest headers write to this slice in separate Goroutines
	defer b.mu.Unlock()
	b.SyncCommitteePeriodsSynced = append(b.SyncCommitteePeriodsSynced, period)
}