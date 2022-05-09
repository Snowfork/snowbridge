package syncer

type BeaconCache struct {
	SyncCommitteePeriodsSynced []uint64
	FinalizedHeaders           []uint64
}

func NewBeaconCache() *BeaconCache {
	return &BeaconCache{
		SyncCommitteePeriodsSynced: []uint64{},
		FinalizedHeaders:           []uint64{},
	}
}
