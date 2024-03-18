package header

import (
	"context"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"github.com/stretchr/testify/require"
	"testing"
)

// Verifies that the closest checkpoint is populated successfully if it is not populated in the first place.
func TestSyncInterimFinalizedUpdate(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}

	client := testutil.MockAPI{}

	store := testutil.MockStore{}

	syncer := syncer.New(&client, settings, &store)

	headerAtSlot4571072, err := testutil.GetHeaderAtSlot(4571072)
	require.NoError(t, err)
	headerAtSlot4571136, err := testutil.GetHeaderAtSlot(4571136)
	require.NoError(t, err)

	//blockAtSlot4563009, err := testutil.GetBlockAtSlot(4563009)
	//require.NoError(t, err)

	client.HeadersBySlot = map[uint64]api.BeaconHeader{
		4571072: headerAtSlot4571072,
		4571136: headerAtSlot4571136,
	}

	client.Header = map[common.Hash]api.BeaconHeader{
		common.HexToHash("0x5119c1f71943a3eea34ddc48c7fe399d4b66f939350036431847ed0913448749"): headerAtSlot4571072,
	}

	/*client.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		4563009: blockAtSlot4563009,
	}*/

	h := Header{
		cache: cache.New(settings.SlotsInEpoch, settings.EpochsPerSyncCommitteePeriod),
		writer: &testutil.MockWriter{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4562496,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		syncer:                       syncer,
		slotsInEpoch:                 settings.SlotsInEpoch,
		epochsPerSyncCommitteePeriod: settings.EpochsPerSyncCommitteePeriod,
	}

	//4571072
	// Find a checkpoint for a slot that is just out of the on-chain synced finalized header block roots range
	err = h.syncInterimFinalizedUpdate(context.Background(), 4570722)
	require.NoError(t, err)
}
