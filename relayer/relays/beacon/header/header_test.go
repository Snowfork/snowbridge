package header

import (
	"context"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/mock"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

// Verifies that the closest checkpoint is populated successfully if it is not populated in the first place.
func TestSyncInterimFinalizedUpdate_WithDataFromAPI(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	p := protocol.New(settings)
	client := mock.API{}
	beaconStore := mock.Store{}

	headerAtSlot4571072, err := testutil.GetHeaderAtSlot(4571072)
	require.NoError(t, err)
	headerAtSlot4571136, err := testutil.GetHeaderAtSlot(4571136)
	require.NoError(t, err)
	headerAtSlot4571137, err := testutil.GetHeaderAtSlot(4571137)
	require.NoError(t, err)
	blockAtSlot4571137, err := testutil.GetBlockAtSlot(4571137)
	require.NoError(t, err)

	client.HeadersBySlot = map[uint64]api.BeaconHeader{
		4571072: headerAtSlot4571072,
		4571136: headerAtSlot4571136,
		4571137: headerAtSlot4571137,
	}
	client.Header = map[common.Hash]api.BeaconHeader{
		common.HexToHash("0x5119c1f71943a3eea34ddc48c7fe399d4b66f939350036431847ed0913448749"): headerAtSlot4571072,
	}
	client.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		4571137: blockAtSlot4571137,
	}

	beaconStates := map[uint64]bool{
		4571072: true,
		4571136: true,
	}
	client.BeaconStates = beaconStates

	h := New(
		&mock.Writer{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4562496,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		&client,
		settings,
		&beaconStore,
		p,
		316,
	)

	// Find a checkpoint for a slot that is just out of the on-chain synced finalized header block roots range
	_, err = h.syncInterimFinalizedUpdate(context.Background(), 4563072, 4571360)
	require.NoError(t, err)
}

func TestSyncInterimFinalizedUpdate_WithDataFromStore(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	p := protocol.New(settings)
	client := mock.API{}
	beaconStore := mock.Store{}

	headerAtSlot4571072, err := testutil.GetHeaderAtSlot(4571072)
	require.NoError(t, err)
	headerAtSlot4571136, err := testutil.GetHeaderAtSlot(4571136)
	require.NoError(t, err)
	headerAtSlot4571137, err := testutil.GetHeaderAtSlot(4571137)
	require.NoError(t, err)
	blockAtSlot4571137, err := testutil.GetBlockAtSlot(4571137)
	require.NoError(t, err)

	client.HeadersBySlot = map[uint64]api.BeaconHeader{
		4571072: headerAtSlot4571072,
		4571136: headerAtSlot4571136,
		4571137: headerAtSlot4571137,
	}
	client.Header = map[common.Hash]api.BeaconHeader{
		common.HexToHash("0x5119c1f71943a3eea34ddc48c7fe399d4b66f939350036431847ed0913448749"): headerAtSlot4571072,
	}
	client.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		4571137: blockAtSlot4571137,
	}

	attestedState, err := testutil.LoadFile("4571136.ssz")
	require.NoError(t, err)
	finalizedState, err := testutil.LoadFile("4571072.ssz")
	require.NoError(t, err)
	// Return the beacon state from the stpore
	beaconStore.StoredBeaconStateData = store.StoredBeaconData{
		AttestedSlot:         4571136,
		FinalizedSlot:        4571072,
		AttestedBeaconState:  attestedState,
		FinalizedBeaconState: finalizedState,
	}

	h := New(
		&mock.Writer{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4562496,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		&client,
		settings,
		&beaconStore,
		p,
		316,
	)

	// Find a checkpoint for a slot that is just out of the on-chain synced finalized header block roots range
	_, err = h.syncInterimFinalizedUpdate(context.Background(), 4563072, 4571360)
	require.NoError(t, err)
}

// Test a scenario where there is a usable beacon update in beacon data store, but it is a different attested and
// finalized state that we calculated to use.
func TestSyncInterimFinalizedUpdate_WithDataFromStoreWithDifferentBlocks(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	p := protocol.New(settings)
	client := mock.API{}
	beaconStore := mock.Store{}

	headerAtSlot4570752, err := testutil.GetHeaderAtSlot(4570752)
	require.NoError(t, err)
	headerAtSlot4570816, err := testutil.GetHeaderAtSlot(4570816)
	require.NoError(t, err)
	headerAtSlot4570818, err := testutil.GetHeaderAtSlot(4570818)
	require.NoError(t, err)
	blockAtSlot4570818, err := testutil.GetBlockAtSlot(4570818)
	require.NoError(t, err)

	client.HeadersBySlot = map[uint64]api.BeaconHeader{
		4570752: headerAtSlot4570752,
		4570816: headerAtSlot4570816,
		4570818: headerAtSlot4570818,
	}
	client.Header = map[common.Hash]api.BeaconHeader{
		common.HexToHash("0x968a372336b4e08a6bbd25e9f31b336d322ede1e5c70763f61d2241ad3d66d36"): headerAtSlot4570752,
	}
	client.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		4570818: blockAtSlot4570818,
	}

	attestedState, err := testutil.LoadFile("4570816.ssz")
	require.NoError(t, err)
	finalizedState, err := testutil.LoadFile("4570752.ssz")
	require.NoError(t, err)
	// Return the beacon state from the store
	beaconStore.StoredBeaconStateData = store.StoredBeaconData{
		AttestedSlot:         4570816,
		FinalizedSlot:        4570752,
		AttestedBeaconState:  attestedState,
		FinalizedBeaconState: finalizedState,
	}

	h := New(
		&mock.Writer{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4562496,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		&client,
		settings,
		&beaconStore,
		p,
		316,
	)

	// Find a checkpoint for a slot that is just out of the on-chain synced finalized header block roots range
	_, err = h.syncInterimFinalizedUpdate(context.Background(), 4563072, 4571360)
	require.NoError(t, err)
}

// Test a scenario where we can get beacon data from the API, but cannot download the beacon state from the API
// or store.
func TestSyncInterimFinalizedUpdate_BeaconStateNotAvailableInAPIAndStore(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	p := protocol.New(settings)
	client := mock.API{}
	beaconStore := mock.Store{}

	headerAtSlot4571072, err := testutil.GetHeaderAtSlot(4571072)
	require.NoError(t, err)
	headerAtSlot4571136, err := testutil.GetHeaderAtSlot(4571136)
	require.NoError(t, err)
	headerAtSlot4571137, err := testutil.GetHeaderAtSlot(4571137)
	require.NoError(t, err)

	client.HeadersBySlot = map[uint64]api.BeaconHeader{
		4571072: headerAtSlot4571072,
		4571136: headerAtSlot4571136,
		4571137: headerAtSlot4571137,
	}

	h := New(
		&mock.Writer{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4562496,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		&client,
		settings,
		&beaconStore,
		p,
		316,
	)

	// Find a checkpoint for a slot that is just out of the on-chain synced finalized header block roots range
	_, err = h.syncInterimFinalizedUpdate(context.Background(), 4570722, 4578922)
	require.Error(t, err)
}

func TestSyncInterimFinalizedUpdate_NoValidBlocksFound(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	p := protocol.New(settings)
	client := mock.API{}
	beaconStore := mock.Store{}

	headerAtSlot4571072, err := testutil.GetHeaderAtSlot(4571072)
	require.NoError(t, err)

	// Only 1 valid header found
	client.HeadersBySlot = map[uint64]api.BeaconHeader{
		4571072: headerAtSlot4571072,
	}

	h := New(
		&mock.Writer{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4562496,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		&client,
		settings,
		&beaconStore,
		p,
		316,
	)

	// Find a checkpoint for a slot that is just out of the on-chain synced finalized header block roots range
	_, err = h.syncInterimFinalizedUpdate(context.Background(), 4570722, 4578922)
	require.Errorf(t, err, "cannot find blocks at boundaries")
}

func TestShouldUpdate(t *testing.T) {
	values := []struct {
		name        string
		apiSlot     uint64
		onChainSlot uint64
		result      bool
	}{
		{
			name:        "should sync, equal to interval",
			apiSlot:     500,
			onChainSlot: 200,
			result:      true,
		},
		{
			name:        "should sync, large gap",
			apiSlot:     800,
			onChainSlot: 200,
			result:      true,
		},
		{
			name:        "should not sync",
			apiSlot:     500,
			onChainSlot: 201,
			result:      false,
		},
	}

	h := Header{}
	h.updateSlotInterval = 300

	for _, tt := range values {
		result := h.shouldUpdate(tt.apiSlot, tt.onChainSlot)
		assert.Equal(t, tt.result, result, "expected %t but found %t", tt.result, result)
	}
}
