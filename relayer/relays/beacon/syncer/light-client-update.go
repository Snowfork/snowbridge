package syncer

type LightClientUpdate struct {
	FinalityHeader BeaconHeader
	SyncCommittee  SyncCommittee
	SyncAggregate  SyncAggregate
}
