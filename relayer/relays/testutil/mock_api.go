package testutil

import (
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
)

type MockAPI struct {
	LatestFinalisedUpdateResponse     api.LatestFinalisedUpdateResponse
	SyncCommitteePeriodUpdateResponse api.SyncCommitteePeriodUpdateResponse
	HeadersBySlot                     map[uint64]api.BeaconHeader
	BlocksAtSlot                      map[uint64]api.BeaconBlockResponse
	Header                            map[common.Hash]api.BeaconHeader
}

func (m *MockAPI) GetBootstrap(blockRoot common.Hash) (api.BootstrapResponse, error) {
	return api.BootstrapResponse{}, nil
}

func (m *MockAPI) GetGenesis() (api.Genesis, error) {
	return api.Genesis{}, nil
}

func (m *MockAPI) GetFinalizedCheckpoint() (api.FinalizedCheckpoint, error) {
	return api.FinalizedCheckpoint{}, nil
}

func (m *MockAPI) GetHeaderBySlot(slot uint64) (api.BeaconHeader, error) {
	value, ok := m.HeadersBySlot[slot]
	if !ok {
		return api.BeaconHeader{}, api.ErrNotFound
	}
	return value, nil
}

func (m *MockAPI) GetHeader(blockRoot common.Hash) (api.BeaconHeader, error) {
	return m.Header[blockRoot], nil
}

func (m *MockAPI) GetBeaconBlockBySlot(slot uint64) (api.BeaconBlockResponse, error) {
	value, ok := m.BlocksAtSlot[slot]
	if !ok {
		return api.BeaconBlockResponse{}, api.ErrNotFound
	}
	return value, nil
}

func (m *MockAPI) GetBeaconBlockRoot(slot uint64) (common.Hash, error) {
	return common.Hash{}, nil
}

func (m *MockAPI) GetBeaconBlock(blockID common.Hash) (api.BeaconBlockResponse, error) {
	return api.BeaconBlockResponse{}, nil
}

func (m *MockAPI) GetSyncCommitteePeriodUpdate(from uint64) (api.SyncCommitteePeriodUpdateResponse, error) {
	return api.SyncCommitteePeriodUpdateResponse{}, nil
}

func (m *MockAPI) GetLatestFinalizedUpdate() (api.LatestFinalisedUpdateResponse, error) {
	return m.LatestFinalisedUpdateResponse, nil
}

func (m *MockAPI) GetBeaconState(stateIdOrSlot string) ([]byte, error) {
	data, err := LoadFile(stateIdOrSlot + ".ssz")
	if err != nil {
		return []byte{}, fmt.Errorf("error reading file: %w", err)
	}
	return data, nil
}
