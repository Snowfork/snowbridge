package testutil

import (
	"context"
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"

	"github.com/ethereum/go-ethereum/common"
)

type MockWriter struct {
	LastFinalizedState state.FinalizedHeader
}

func (m *MockWriter) GetSecondLastFinalizedSlot() (types.U32, error) {
	return 0, nil
}

func (m *MockWriter) GetLastFinalizedStateIndex() (types.U32, error) {
	return 0, nil
}

func (m *MockWriter) GetFinalizedBeaconRootByIndex(index uint32) (types.H256, error) {
	return types.H256{}, nil
}

func (m *MockWriter) BatchCall(ctx context.Context, extrinsic string, calls []interface{}) error {
	return nil
}

func (m *MockWriter) WriteToParachainAndRateLimit(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	return nil
}
func (m *MockWriter) WriteToParachainAndWatch(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	update, ok := payload[0].(scale.UpdatePayload)
	if ok {
		m.LastFinalizedState.BeaconSlot = uint64(update.FinalizedHeader.Slot)
		htr, err := update.FinalizedHeader.ToSSZ().HashTreeRoot()
		if err != nil {
			return fmt.Errorf("hash tree root error")
		}
		m.LastFinalizedState.BeaconBlockRoot = htr
	} else {
		return fmt.Errorf("type conversion error")
	}
	return nil
}

func (m *MockWriter) GetLastFinalizedHeaderState() (state.FinalizedHeader, error) {
	return m.LastFinalizedState, nil
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

func (m *MockWriter) FindCheckPointBackward(slot uint64) (state.FinalizedHeader, error) {
	return state.FinalizedHeader{}, nil
}
