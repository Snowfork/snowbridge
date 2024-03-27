package syncer

import (
	"encoding/json"
	"fmt"
	"strconv"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/mock"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const TestUrl = "https://lodestar-sepolia.chainsafe.io"

func newTestRunner() *Syncer {
	return New(api.NewBeaconClient(TestUrl), &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}))
}

// Verifies that the Lodestar provided finalized endpoint matches the manually constructed finalized endpoint
func TestGetFinalizedUpdateAtSlot(t *testing.T) {
	t.Skip("skip testing utility test")

	syncer := newTestRunner()

	// Get lodestar finalized update
	lodestarUpdate, err := syncer.GetFinalizedUpdate()
	require.NoError(t, err)
	lodestarUpdateJSON := lodestarUpdate.Payload.ToJSON()

	attestedSlot, err := syncer.FindLatestAttestedHeadersAtInterval(uint64(lodestarUpdate.Payload.AttestedHeader.Slot), 9331)
	require.NoError(t, err)

	// Manually construct the finalized update for the same block
	manualUpdate, err := syncer.GetLatestPossibleFinalizedUpdate(attestedSlot, 9331)
	require.NoError(t, err)
	manualUpdateJSON := manualUpdate.Payload.ToJSON()

	lodestarPayload, err := json.Marshal(lodestarUpdateJSON)
	require.NoError(t, err)
	manualPayload, err := json.Marshal(manualUpdateJSON)
	require.NoError(t, err)

	// The JSON should be same
	require.JSONEq(t, string(lodestarPayload), string(manualPayload))
}

// Verifies that the Lodestar provided finalized endpoint matches the manually constructed finalized endpoint
func TestGetFinalizedUpdateWithSyncCommitteeUpdateAtSlot(t *testing.T) {
	t.Skip("skip testing utility test")

	beaconData4645280, err := testutil.LoadFile("4645280.ssz")
	require.NoError(t, err)
	beaconData4644864, err := testutil.LoadFile("4644864.ssz")
	require.NoError(t, err)
	beaconData4644928, err := testutil.LoadFile("4644928.ssz")
	require.NoError(t, err)

	syncer := New(api.NewBeaconClient(TestUrl), &mock.Store{
		BeaconStateData: map[uint64][]byte{
			4645280: beaconData4645280,
			//4644864: beaconData4644864,
			//4644928: beaconData4644928,
		},
		StoredBeaconStateData: store.StoredBeaconData{
			AttestedSlot:         4644864,
			FinalizedSlot:        4644928,
			AttestedBeaconState:  beaconData4644864,
			FinalizedBeaconState: beaconData4644928,
		},
	}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}))

	syncCommitteePeriod := uint64(567)
	// Get lodestar finalized update
	lodestarUpdate, err := syncer.GetSyncCommitteePeriodUpdate(syncCommitteePeriod)
	require.NoError(t, err)
	lodestarUpdateJSON := lodestarUpdate.Payload.ToJSON()

	// Manually construct a finalized update
	manualUpdate, err := syncer.GetFinalizedUpdateWithSyncCommittee(syncCommitteePeriod)
	require.NoError(t, err)
	manualUpdateJSON := manualUpdate.Payload.ToJSON()

	lodestarPayload, err := json.Marshal(lodestarUpdateJSON.NextSyncCommitteeUpdate.NextSyncCommittee.Pubkeys)
	require.NoError(t, err)
	manualPayload, err := json.Marshal(manualUpdateJSON.NextSyncCommitteeUpdate.NextSyncCommittee.Pubkeys)
	require.NoError(t, err)

	// The JSON should be same
	require.JSONEq(t, string(lodestarPayload), string(manualPayload))
	require.Equal(t, lodestarUpdateJSON.NextSyncCommitteeUpdate.NextSyncCommittee.AggregatePubkey, manualUpdateJSON.NextSyncCommitteeUpdate.NextSyncCommittee.AggregatePubkey)
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
	mockAPI := mock.API{}

	mockAPI.HeadersBySlot = map[uint64]api.BeaconHeader{
		8160: {Slot: 8160},
		// skip 8128
		// skip 8096
		8064: {Slot: 8064}, // this should be the first valid attested header
		// skip 8032
		8000: {Slot: 8000},
	}

	syncer := New(&mockAPI, &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}))

	attested, err := syncer.FindLatestAttestedHeadersAtInterval(8192, 100)
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

	syncer = New(&mockAPI, &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}))

	attested, err = syncer.FindLatestAttestedHeadersAtInterval(32768, 25076)
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

	syncer = New(&mockAPI, &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}))

	attested, err = syncer.FindLatestAttestedHeadersAtInterval(32768, 25076)
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

	syncer = New(&mockAPI, &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}))

	attested, err = syncer.FindLatestAttestedHeadersAtInterval(32768, 32540)
	assert.Error(t, err)
}
