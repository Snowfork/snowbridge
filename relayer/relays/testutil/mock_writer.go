package testutil

import (
	"context"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"

	"github.com/ethereum/go-ethereum/common"
)

type MockWriter struct {
	LastFinalizedState state.FinalizedHeader
}

func (m *MockWriter) BatchCall(ctx context.Context, extrinsic string, calls []interface{}) error {
	return nil
}

func (m *MockWriter) WriteToParachainAndRateLimit(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	return nil
}
func (m *MockWriter) WriteToParachainAndWatch(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	return nil
}

func (m *MockWriter) GetLastFinalizedHeaderState() (state.FinalizedHeader, error) {
	return m.LastFinalizedState, nil
}

func (m *MockWriter) GetLastExecutionHeaderState() (state.ExecutionHeader, error) {
	return state.ExecutionHeader{}, nil
}

func (m *MockWriter) GetFinalizedStateByStorageKey(key string) (scale.BeaconState, error) {
	return scale.BeaconState{}, nil
}

func (m *MockWriter) GetLastBasicChannelBlockNumber() (uint64, error) {
	return 0, nil
}

func (m *MockWriter) GetLastBasicChannelNonceByAddress(address common.Address) (uint64, error) {
	return 0, nil

}
func (m *MockWriter) GetFinalizedHeaderStateByBlockRoot(blockRoot types.H256) (state.FinalizedHeader, error) {
	return state.FinalizedHeader{}, nil
}
