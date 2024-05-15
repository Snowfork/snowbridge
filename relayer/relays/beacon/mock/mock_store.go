package mock

import (
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
)

type Store struct {
	StoredBeaconStateData store.StoredBeaconData
	BeaconStateData       map[uint64][]byte
}

func (m *Store) FindBeaconStateWithinRange(slot, boundary uint64) (store.StoredBeaconData, error) {
	return m.StoredBeaconStateData, nil
}

func (m *Store) WriteEntry(attestedSlot, finalizedSlot uint64, attestedStateData, finalizedStateData []byte) error {
	return nil
}

func (m *Store) GetBeaconStateData(slot uint64) ([]byte, error) {
	value, ok := m.BeaconStateData[slot]
	if !ok {
		return nil, api.ErrNotFound
	}
	return value, nil
}

func (m *Store) Connect() error {
	return nil
}

func (m *Store) Close() {

}
