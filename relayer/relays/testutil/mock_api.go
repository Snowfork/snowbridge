package testutil

import (
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
)

type MockAPI struct {
	LatestFinalisedUpdateResponse     api.LatestFinalisedUpdateResponse
	SyncCommitteePeriodUpdateResponse api.SyncCommitteePeriodUpdateResponse
	HeadersAtSlot                     map[uint64]api.BeaconHeader
	BlocksAtSlot                      map[uint64]api.BeaconBlockResponse
	Header                            api.BeaconHeader
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
	return m.HeadersAtSlot[slot], nil
}

func (m *MockAPI) GetHeader(blockRoot common.Hash) (api.BeaconHeader, error) {
	return m.Header, nil
}

func (m *MockAPI) GetBeaconBlockBySlot(slot uint64) (api.BeaconBlockResponse, error) {
	return m.BlocksAtSlot[slot], nil
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
	if stateIdOrSlot == "4563008" {
		data, err := LoadFile("4563008.ssz")
		if err != nil {
			return []byte{}, fmt.Errorf("error reading file: %w", err)
		}
		return data, nil
	}
	if stateIdOrSlot == "4562944" {
		data, err := LoadFile("4562944.ssz")
		if err != nil {
			return []byte{}, fmt.Errorf("error reading file: %w", err)
		}
		return data, nil
	}
	return []byte{}, nil
}
