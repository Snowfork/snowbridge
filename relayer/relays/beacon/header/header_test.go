package header

import (
	"context"
	"encoding/json"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"testing"
)

const TestUrl = "http://localhost:3500"

// Verifies that the closest checkpoint is populated successfully if it is not populated in the first place.
func TestPopulateClosestCheckpoint(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}

	var finalizedHeader api.BeaconHeader

	data, err := testutil.LoadFile("header_4562944.json")
	require.NoError(t, err)

	err = json.Unmarshal(data, &finalizedHeader)
	require.NoError(t, err)

	client := testutil.MockAPI{
		Header: finalizedHeader,
	}

	store := testutil.MockStore{}

	syncer := syncer.New(&client, settings, &store)

	headerAtSlot4563008, err := testutil.GetHeaderAtSlot(4563008)
	require.NoError(t, err)
	headerAtSlot4563009, err := testutil.GetHeaderAtSlot(4563009)
	require.NoError(t, err)
	blockAtSlot4563009, err := testutil.GetBlockAtSlot(4563009)
	require.NoError(t, err)

	client.HeadersAtSlot = map[uint64]api.BeaconHeader{
		4563008: headerAtSlot4563008,
		4563009: headerAtSlot4563009,
	}

	client.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		4563009: blockAtSlot4563009,
	}

	h := Header{
		cache: cache.New(settings.SlotsInEpoch, settings.EpochsPerSyncCommitteePeriod),
		writer: &testutil.MockWriter{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4565856,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		syncer:                       syncer,
		slotsInEpoch:                 settings.SlotsInEpoch,
		epochsPerSyncCommitteePeriod: settings.EpochsPerSyncCommitteePeriod,
	}

	// Find a checkpoint for a slot that is just out of the on-chain synced finalized header block roots range
	proof, err := h.populateClosestCheckpoint(context.Background(), 4557600) // 4565856 - 8192 - 64
	assert.Equal(t, proof.Slot, uint64(4562944))

	require.NoError(t, err)
}
