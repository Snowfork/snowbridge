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

	"github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const TestUrl = "https://lodestar-sepolia.chainsafe.io"
const MaxRedundancy = 20

func newTestRunner() *Syncer {
	return New(api.NewBeaconClient(TestUrl, TestUrl), &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
	}, MaxRedundancy))
}

// Verifies that the Lodestar provided finalized endpoint matches the manually constructed finalized endpoint
func TestGetFinalizedUpdateAtSlot(t *testing.T) {
	t.Skip("skip testing utility test")

	syncer := newTestRunner()

	// Get lodestar finalized update
	lodestarUpdate, err := syncer.GetFinalizedUpdate()
	require.NoError(t, err)
	lodestarUpdateJSON := lodestarUpdate.Payload.ToJSON()

	attestedSlot, err := syncer.FindValidAttestedHeader(uint64(lodestarUpdate.Payload.AttestedHeader.Slot), 9331)
	require.NoError(t, err)

	// Manually construct the finalized update for the same block
	manualUpdate, err := syncer.GetFinalizedUpdateAtAttestedSlot(attestedSlot, 9331, false)
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

	beaconData64, err := testutil.LoadFile("64.ssz")
	require.NoError(t, err)
	beaconData129, err := testutil.LoadFile("129.ssz")
	require.NoError(t, err)

	headerAtSlot64, err := testutil.GetHeaderAtSlot(64)
	require.NoError(t, err)
	headerAtSlot129, err := testutil.GetHeaderAtSlot(129)
	require.NoError(t, err)
	headerAtSlot130, err := testutil.GetHeaderAtSlot(130)
	require.NoError(t, err)

	blockAtSlot, err := testutil.GetBlockAtSlot(130)
	require.NoError(t, err)

	syncCommitteeUpdate, err := testutil.GetSyncCommitteeUpdate(0)
	require.NoError(t, err)

	mockAPI := mock.API{
		LatestFinalisedUpdateResponse:     api.LatestFinalisedUpdateResponse{},
		SyncCommitteePeriodUpdateResponse: syncCommitteeUpdate,
		HeadersBySlot: map[uint64]api.BeaconHeader{
			64:  headerAtSlot64,
			129: headerAtSlot129,
			130: headerAtSlot130,
		},
		BlocksAtSlot: map[uint64]api.BeaconBlockResponse{
			130: blockAtSlot,
		},
		Header: map[common.Hash]api.BeaconHeader{
			common.HexToHash("0x3d0145a0f4565ac6fde12d4a4e7f5df35bec009ee9cb30abaac2eaab8de0d6c5"): headerAtSlot64,
		},
		BeaconStates: nil,
	}

	syncer := New(&mockAPI, &mock.Store{
		BeaconStateData: map[uint64][]byte{
			64:  beaconData64,
			129: beaconData129,
		},
		StoredBeaconStateData: store.StoredBeaconData{
			AttestedSlot:         129,
			FinalizedSlot:        64,
			AttestedBeaconState:  beaconData129,
			FinalizedBeaconState: beaconData64,
		},
	}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
	}, MaxRedundancy))

	// Manually construct a finalized update
	manualUpdate, err := syncer.GetFinalizedUpdateAtAttestedSlot(129, 0, true)
	require.NoError(t, err)
	manualUpdateJSON := manualUpdate.Payload.ToJSON()

	lodestarPayload, err := testutil.LoadFile("sync_committee_comp.json")
	require.NoError(t, err)
	manualPayload, err := json.Marshal(manualUpdateJSON)
	require.NoError(t, err)

	require.Equal(t, lodestarPayload, manualPayload)
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
		8065: {Slot: 8065}, // next header so that we can get the sync aggregate
		// skip 8032
		8000: {Slot: 8000},
	}

	mockAPI.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		8065: {
			Data: api.BeaconBlockResponseData{Message: api.BeaconBlockResponseMessage{Body: api.BeaconBlockResponseBody{SyncAggregate: api.SyncAggregateResponse{
				SyncCommitteeBits:      "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00000000000000000000000000000000",
				SyncCommitteeSignature: "0x946646f0dacd480ecb8878709e7632037fd1adc7c99f15cf725ecd9f3710aa848de8f9fa9595479547065e76bb018d75077fc1912908c9d50e254e99db192b1a76ed1b2cfffafb92742334230787cb94447897148cee37053d4e682c85149b27",
			}}}},
		},
	}

	syncer := New(&mockAPI, &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
	}, MaxRedundancy))

	attested, err := syncer.FindValidAttestedHeader(8000, 8160)
	assert.NoError(t, err)
	assert.Equal(t, "8064", strconv.FormatUint(attested, 10))

	mockAPI.HeadersBySlot = map[uint64]api.BeaconHeader{
		// skip 32768
		32736: {Slot: 32736},
		32704: {Slot: 32704},
		32705: {Slot: 32705}, // next header so that we can get the sync aggregate
		// skip 32672
		32640: {Slot: 32640},
		32608: {Slot: 32608},
		32576: {Slot: 32576},
	}

	mockAPI.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		32705: {
			Data: api.BeaconBlockResponseData{Message: api.BeaconBlockResponseMessage{Body: api.BeaconBlockResponseBody{SyncAggregate: api.SyncAggregateResponse{
				SyncCommitteeBits:      "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00000000000000000000000000000000",
				SyncCommitteeSignature: "0x946646f0dacd480ecb8878709e7632037fd1adc7c99f15cf725ecd9f3710aa848de8f9fa9595479547065e76bb018d75077fc1912908c9d50e254e99db192b1a76ed1b2cfffafb92742334230787cb94447897148cee37053d4e682c85149b27",
			}}}},
		},
	}

	syncer = New(&mockAPI, &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
	}, MaxRedundancy))

	attested, err = syncer.FindValidAttestedHeader(32576, 32704)
	assert.NoError(t, err)
	assert.Equal(t, "32704", strconv.FormatUint(attested, 10))

	mockAPI.HeadersBySlot = map[uint64]api.BeaconHeader{
		// skip 32768
		32736: {Slot: 32736},
		32704: {Slot: 32704},
		32705: {Slot: 32705}, // next header so that we can get the sync aggregate
		// skip 32672
		32640: {Slot: 32640},
		// skip 32608
		32576: {Slot: 32576},
	}

	mockAPI.BlocksAtSlot = map[uint64]api.BeaconBlockResponse{
		32705: {
			Data: api.BeaconBlockResponseData{Message: api.BeaconBlockResponseMessage{Body: api.BeaconBlockResponseBody{SyncAggregate: api.SyncAggregateResponse{
				SyncCommitteeBits:      "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00000000000000000000000000000000",
				SyncCommitteeSignature: "0x946646f0dacd480ecb8878709e7632037fd1adc7c99f15cf725ecd9f3710aa848de8f9fa9595479547065e76bb018d75077fc1912908c9d50e254e99db192b1a76ed1b2cfffafb92742334230787cb94447897148cee37053d4e682c85149b27",
			}}}},
		},
	}

	syncer = New(&mockAPI, &mock.Store{}, protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
	}, MaxRedundancy))

	attested, err = syncer.FindValidAttestedHeader(25076, 32736)
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
	}, MaxRedundancy))

	attested, err = syncer.FindValidAttestedHeader(32540, 32768)
	assert.Error(t, err)
}
