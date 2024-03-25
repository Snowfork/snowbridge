package testutil

import (
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
)

type MockStore struct {
	BeaconStateData store.StoredBeaconData
}

func (m *MockStore) Connect() error {
	return nil
}

func (m *MockStore) Close() {

}

func (m *MockStore) StoreUpdate(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod uint64) error {
	return nil
}

func (m *MockStore) FindBeaconStateWithinSyncPeriodRange(slot, boundary uint64, findMax bool) (store.StoredBeaconData, error) {
	return m.BeaconStateData, nil
}
