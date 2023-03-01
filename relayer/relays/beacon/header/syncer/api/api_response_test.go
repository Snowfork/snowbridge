package api

import (
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/stretchr/testify/require"
	"os"
	"testing"
)

func TestExecutionPayloadToScale(t *testing.T) {
	data, err := os.ReadFile("execution_payload.ssz")
	require.NoError(t, err)

	beaconBlock := state.BeaconBlockBellatrix{}

	err = beaconBlock.UnmarshalSSZ(data)
	require.NoError(t, err)
}
