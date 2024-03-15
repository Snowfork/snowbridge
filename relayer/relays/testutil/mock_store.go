package testutil

import "github.com/snowfork/snowbridge/relayer/relays/beacon/store"

type MockStore struct {
}

func (m *MockStore) Connect() error {
	return nil
}

func (m *MockStore) Close() {

}

func (m *MockStore) StoreUpdate(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod uint64) error {
	return nil
}

func (m *MockStore) FindBeaconStateWithinSyncPeriodRange(baseSlot, slotRange uint64) (store.StoredBeaconData, error) {
	return store.StoredBeaconData{}, nil
}
