package header

import (
	"context"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"github.com/stretchr/testify/require"
)

const TestUrl = "http://localhost:3500"

// Verifies that the closest checkpoint is populated successfully if it is not populated in the first place.
func TestPopulateClosestCheckpoint(t *testing.T) {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}

	syncCommittee, err := testutil.GetSyncCommitteeUpdate()
	require.NoError(t, err)

	finalizedUpdate, err := testutil.GetFinalizedUpdate()
	require.NoError(t, err)

	client := testutil.MockAPI{
		LatestFinalisedUpdateResponse: finalizedUpdate,
		//SyncCommitteePeriodUpdateResponse: syncCommittee,
	}

	syncer := syncer.New(&client, settings)
	
	slot, err := util.ToUint64(syncCommittee.Data.AttestedHeader.Beacon.Slot)
	require.NoError(t, err)

	checkpointSlot := syncer.CalculateNextCheckpointSlot(slot)

	headerAtSlot, err := testutil.GetHeaderAtSlot(checkpointSlot)
	require.NoError(t, err)

	client.HeaderAtSlot = headerAtSlot

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
		syncer:                       syncer,
		slotsInEpoch:                 settings.SlotsInEpoch,
		epochsPerSyncCommitteePeriod: settings.EpochsPerSyncCommitteePeriod,
	}

	syncCommitteeSlot, err := util.ToUint64(syncCommittee.Data.AttestedHeader.Beacon.Slot)

	_, err = h.populateClosestCheckpoint(context.Background(), syncCommitteeSlot)

	require.NoError(t, err)
}
