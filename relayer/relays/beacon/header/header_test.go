package header

import (
	"context"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"github.com/stretchr/testify/require"
)

const TestUrl = "http://localhost:3500"

// Verfies that the closest checkpoint is populated successfully if it is not populated in the first place.
func TestPopulateClosestCheckpoint(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	client := api.NewBeaconClient(TestUrl, settings.SlotsInEpoch)
	h := Header{
		cache: cache.New(settings.SlotsInEpoch, settings.EpochsPerSyncCommitteePeriod),
		writer: &testutil.MockWriter{
			LastFinalizedState: state.FinalizedHeader{
				BeaconBlockRoot:       common.Hash{},
				BeaconSlot:            4555872,
				InitialCheckpointRoot: common.Hash{},
				InitialCheckpointSlot: 0,
			},
		},
		syncer:                       syncer.New(client, settings),
		slotsInEpoch:                 settings.SlotsInEpoch,
		epochsPerSyncCommitteePeriod: settings.EpochsPerSyncCommitteePeriod,
	}

	_, err := h.populateClosestCheckpoint(context.Background(), 4555659)
	require.NoError(t, err)
}
