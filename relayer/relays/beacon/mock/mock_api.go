package mock

import (
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

type API struct {
	LatestFinalisedUpdateResponse     api.LatestFinalisedUpdateResponse
	SyncCommitteePeriodUpdateResponse api.SyncCommitteePeriodUpdateResponse
	HeadersBySlot                     map[uint64]api.BeaconHeader
	BlocksAtSlot                      map[uint64]api.BeaconBlockResponse
	Header                            map[common.Hash]api.BeaconHeader
	BeaconStates                      map[uint64]bool
}

func (m *API) GetHeaderAtHead() (api.BeaconHeader, error) {
	return api.BeaconHeader{}, nil
}

func (m *API) GetBootstrap(blockRoot common.Hash) (api.BootstrapResponse, error) {
	return api.BootstrapResponse{}, nil
}

func (m *API) GetGenesis() (api.Genesis, error) {
	return api.Genesis{}, nil
}

func (m *API) GetFinalizedCheckpoint() (api.FinalizedCheckpoint, error) {
	return api.FinalizedCheckpoint{}, nil
}

func (m *API) GetHeaderBySlot(slot uint64) (api.BeaconHeader, error) {
	value, ok := m.HeadersBySlot[slot]
	if !ok {
		return api.BeaconHeader{}, api.ErrNotFound
	}
	return value, nil
}

func (m *API) GetHeaderByBlockRoot(blockRoot common.Hash) (api.BeaconHeader, error) {
	return m.Header[blockRoot], nil
}

func (m *API) GetBeaconBlockBySlot(slot uint64) (api.BeaconBlockResponse, error) {
	value, ok := m.BlocksAtSlot[slot]
	if !ok {
		return api.BeaconBlockResponse{}, api.ErrNotFound
	}
	return value, nil
}

func (m *API) GetBeaconBlockBytes(blockRoot common.Hash) ([]byte, error) {
	return nil, nil
}

func (m *API) GetBeaconBlockRoot(slot uint64) (common.Hash, error) {
	return common.Hash{}, nil
}

func (m *API) GetBeaconBlock(blockID common.Hash) (api.BeaconBlockResponse, error) {
	return api.BeaconBlockResponse{}, nil
}

func (m *API) GetSyncCommitteePeriodUpdate(from uint64) (api.SyncCommitteePeriodUpdateResponse, error) {
	return m.SyncCommitteePeriodUpdateResponse, nil
}

func (m *API) GetLatestFinalizedUpdate() (api.LatestFinalisedUpdateResponse, error) {
	return m.LatestFinalisedUpdateResponse, nil
}

func (m *API) GetBeaconState(stateIdOrSlot string) ([]byte, error) {
	slot, err := util.ToUint64(stateIdOrSlot)
	if err != nil {
		return nil, fmt.Errorf("invalid beacon state slot: %w", err)
	}

	_, ok := m.BeaconStates[slot]
	if !ok {
		return nil, api.ErrNotFound
	}

	data, err := testutil.LoadFile(stateIdOrSlot + ".ssz")
	if err != nil {
		return nil, fmt.Errorf("error reading file: %w", err)
	}
	return data, nil
}
