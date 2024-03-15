package syncer

import (
	"encoding/json"
	"fmt"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"strconv"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const TestUrl = "https://lodestar-sepolia.chainsafe.io"

func newTestRunner() *Syncer {
	return New(api.NewBeaconClient(TestUrl, 32), config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}, &testutil.MockStore{})
}

// Verifies that the Lodestar provided finalized endpoint matches the manually constructed finalized endpoint
func TestGetFinalizedUpdateAtSlot(t *testing.T) {
	t.Skip("skip testing utility test")

	syncer := newTestRunner()

	// Get lodestar finalized update
	lodestarUpdate, err := syncer.GetFinalizedUpdate()
	require.NoError(t, err)
	lodestarUpdateJSON := lodestarUpdate.Payload.ToJSON()

	// Manually construct the finalized update for the same block
	manualUpdate, err := syncer.GetFinalizedUpdateAtAttestedSlot(uint64(lodestarUpdate.Payload.AttestedHeader.Slot), 9331)
	require.NoError(t, err)
	manualUpdateJSON := manualUpdate.Payload.ToJSON()

	lodestarPayload, err := json.Marshal(lodestarUpdateJSON)
	require.NoError(t, err)
	manualPayload, err := json.Marshal(manualUpdateJSON)
	require.NoError(t, err)

	// The JSON should be same
	require.JSONEq(t, string(lodestarPayload), string(manualPayload))
}

func TestGetInitialCheckpoint(t *testing.T) {
	t.Skip("skip testing utility test")

	syncer := newTestRunner()

	response, err := syncer.GetCheckpoint()
	assert.NoError(t, err)
	jsonUpdate := response.ToJSON()

	j, err := json.MarshalIndent(jsonUpdate, "", "  ")
	assert.NoError(t, err)
	fmt.Println(string(j))
}

func TestFindAttestedAndFinalizedHeadersAtBoundary(t *testing.T) {
	mockAPI := testutil.MockAPI{}

	mockAPI.HeadersBySlot = map[uint64]api.BeaconHeader{
		8160: {Slot: 8160},
		// skip 8128
		// skip 8096
		8064: {Slot: 8064}, // this should be the first valid attested header
		// skip 8032
		8000: {Slot: 8000},
	}

	syncer := New(&mockAPI, config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}, &testutil.MockStore{})

	attested, err := syncer.findAttestedAndFinalizedHeadersAtBoundary(8192, 100)
	assert.NoError(t, err)
	assert.Equal(t, "8064", strconv.FormatUint(attested, 10))

	mockAPI.HeadersBySlot = map[uint64]api.BeaconHeader{
		// skip 32768
		32736: {Slot: 32736},
		32704: {Slot: 32704},
		// skip 32672
		32640: {Slot: 32640},
		32608: {Slot: 32608},
		32576: {Slot: 32576},
	}

	syncer = New(&mockAPI, config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}, &testutil.MockStore{})

	attested, err = syncer.findAttestedAndFinalizedHeadersAtBoundary(32768, 25076)
	assert.NoError(t, err)
	assert.Equal(t, "32704", strconv.FormatUint(attested, 10))

	mockAPI.HeadersBySlot = map[uint64]api.BeaconHeader{
		// skip 32768
		32736: {Slot: 32736},
		32704: {Slot: 32704},
		// skip 32672
		32640: {Slot: 32640},
		// skip 32608
		32576: {Slot: 32576},
	}

	syncer = New(&mockAPI, config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}, &testutil.MockStore{})

	attested, err = syncer.findAttestedAndFinalizedHeadersAtBoundary(32768, 25076)
	assert.NoError(t, err)
	assert.Equal(t, "32704", strconv.FormatUint(attested, 10))

	mockAPI.HeadersBySlot = map[uint64]api.BeaconHeader{
		// skip 32768
		32736: {Slot: 32736},
		32704: {Slot: 32704},
		// skip 32672
		// skip 32640
		// skip 32608
		// skip 32576
		// skip 32544
		32512: {Slot: 32512},
		32480: {Slot: 32480},
		32448: {Slot: 32448},
	}

	syncer = New(&mockAPI, config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}, &testutil.MockStore{})

	attested, err = syncer.findAttestedAndFinalizedHeadersAtBoundary(32768, 32540)
	assert.Error(t, err)
}
