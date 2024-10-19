package mock

import (
	"context"
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"

	"github.com/ethereum/go-ethereum/common"
)

type Writer struct {
	LastFinalizedState              state.FinalizedHeader
	LastFinalizedStateIndex         types.U32
	FinalizedBeaconRootByIndex      map[uint32]types.H256
	FinalizedHeaderStateByBlockRoot map[types.H256]state.FinalizedHeader
}

func (m *Writer) GetLastExecutionHeaderState() (state.ExecutionHeader, error) {
	return state.ExecutionHeader{}, nil
}

func (m *Writer) GetLastFinalizedStateIndex() (types.U32, error) {
	return m.LastFinalizedStateIndex, nil
}

func (m *Writer) GetFinalizedBeaconRootByIndex(index uint32) (types.H256, error) {
	return m.FinalizedBeaconRootByIndex[index], nil
}

func (m *Writer) BatchCall(ctx context.Context, extrinsic []string, calls []interface{}) error {
	return nil
}

func (m *Writer) WriteToParachainAndRateLimit(ctx context.Context, extrinsicName string, payload ...interface{}) error {
	return nil
}
func (m *Writer) WriteToParachainAndWatch(ctx context.Context, extrinsicName string, payload ...interface{}) error {
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

func (m *Writer) GetLastFinalizedHeaderState() (state.FinalizedHeader, error) {
	return m.LastFinalizedState, nil
}

func (m *Writer) GetFinalizedStateByStorageKey(key string) (scale.BeaconState, error) {
	return scale.BeaconState{}, nil
}

func (m *Writer) GetLastBasicChannelBlockNumber() (uint64, error) {
	return 0, nil
}

func (m *Writer) GetLastBasicChannelNonceByAddress(address common.Address) (uint64, error) {
	return 0, nil

}
func (m *Writer) GetFinalizedHeaderStateByBlockRoot(blockRoot types.H256) (state.FinalizedHeader, error) {
	return m.FinalizedHeaderStateByBlockRoot[blockRoot], nil
}

func (m *Writer) FindCheckPointBackward(slot uint64) (state.FinalizedHeader, error) {
	return state.FinalizedHeader{}, nil
}
