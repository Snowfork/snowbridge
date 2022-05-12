package syncer

type BeaconCache struct {
	SyncCommitteePeriodsSynced []uint64
	FinalizedHeaders           []uint64
}

func NewBeaconCache() *BeaconCache {
	return &BeaconCache{
		SyncCommitteePeriodsSynced: []uint64{},
		FinalizedHeaders:           []uint64{}, // TODO rather cache by block root, than slot. Need SSZ lib to do that.
	}
}
