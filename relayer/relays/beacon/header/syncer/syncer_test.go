package syncer

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const TestUrl = "https://lodestar-sepolia.chainsafe.io"

func TestCalculateNextCheckpointSlot(t *testing.T) {
	values := []struct {
		name     string
		slot     uint64
		expected uint64
	}{
		{
			name:     "slot 41",
			slot:     41,
			expected: 64,
		},
		{
			name:     "slot 64",
			slot:     64,
			expected: 64,
		},
		{
			name:     "slot 78",
			slot:     78,
			expected: 128,
		},
	}

	syncer := Syncer{}
	syncer.setting.SlotsInEpoch = 8
	syncer.setting.EpochsPerSyncCommitteePeriod = 8

	for _, tt := range values {
		result := syncer.CalculateNextCheckpointSlot(tt.slot)
		assert.Equal(t, tt.expected, result, "expected %t but found %t for slot %d", tt.expected, result, tt.slot)
	}
}

func newTestRunner() *Syncer {
	return New(api.NewBeaconClient(TestUrl, 32), config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	})
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
	manualUpdate, err := syncer.GetFinalizedUpdateAtAttestedSlot(uint64(lodestarUpdate.Payload.AttestedHeader.Slot))
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
