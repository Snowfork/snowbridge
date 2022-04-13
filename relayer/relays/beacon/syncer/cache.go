package syncer

type BeaconCache struct {
	SyncCommitteePeriodsSynced map[uint64][]bool
}

func NewBeaconCache() *BeaconCache {
	return &BeaconCache{
		SyncCommitteePeriodsSynced: make(map[uint64][]bool), 
	}
}
