package testutil

import (
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"

	"github.com/ethereum/go-ethereum/common"
)

type MockAPI struct {
	LatestFinalisedUpdateResponse     api.LatestFinalisedUpdateResponse
	SyncCommitteePeriodUpdateResponse api.SyncCommitteePeriodUpdateResponse
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
	return api.BeaconHeader{}, nil
}

func (m *MockAPI) GetHeader(blockRoot common.Hash) (api.BeaconHeader, error) {
	return api.BeaconHeader{}, nil
}

func (m *MockAPI) GetBeaconBlockBySlot(slot uint64) (api.BeaconBlockResponse, error) {
	return api.BeaconBlockResponse{}, nil
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
	return api.LatestFinalisedUpdateResponse{}, nil
}

func (m *MockAPI) GetBeaconState(stateIdOrSlot string) ([]byte, error) {
	return []byte{}, nil
}
