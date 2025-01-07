package store

import (
	"fmt"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/testutil"
	"github.com/stretchr/testify/require"
	"os"
	"testing"

	_ "github.com/mattn/go-sqlite3"
)

const TestDataStoreFile = "./"
const MaxRedundancy = 20

func TestGetBeaconState(t *testing.T) {
	_ = os.RemoveAll(TestDataStoreFile + BeaconStateDir)
	_ = os.Remove(TestDataStoreFile + BeaconStoreName)

	specSettings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		ForkVersions: config.ForkVersions{
			Deneb:   0,
			Electra: 800000,
		},
	}
	store := New(TestDataStoreFile, 100, *protocol.New(specSettings, MaxRedundancy))

	err := store.Connect()
	require.NoError(t, err)
	defer func() {
		err := os.RemoveAll(TestDataStoreFile + BeaconStateDir)
		require.NoError(t, err)
		err = os.Remove(TestDataStoreFile + BeaconStoreName)
		require.NoError(t, err)
		store.Close()
	}()

	attestedSlot := uint64(4570816)
	finalizedSlot := uint64(4570752)

	attestedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", attestedSlot))
	require.NoError(t, err)
	finalizedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", finalizedSlot))
	require.NoError(t, err)

	err = store.WriteEntry(attestedSlot, finalizedSlot, attestedState, finalizedState)
	require.NoError(t, err)

	// Check that we can get the beacon states
	attestedStateStore, err := store.GetBeaconStateData(attestedSlot)
	require.NoError(t, err)
	require.Equal(t, attestedState, attestedStateStore)

	finalizedStateStore, err := store.GetBeaconStateData(finalizedSlot)
	require.NoError(t, err)
	require.Equal(t, finalizedState, finalizedStateStore)

	// Check that a non-existent state returns an error
	_, err = store.GetBeaconStateData(35345345)
	require.Error(t, err)
}

func TestPruneOldStates(t *testing.T) {
	_ = os.RemoveAll(TestDataStoreFile + BeaconStateDir)
	_ = os.Remove(TestDataStoreFile + BeaconStoreName)

	store := New(TestDataStoreFile, 2, *protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		ForkVersions: config.ForkVersions{
			Deneb:   0,
			Electra: 800000,
		},
	}, MaxRedundancy))
	err := store.Connect()
	require.NoError(t, err)
	defer func() {
		err := os.RemoveAll(TestDataStoreFile + BeaconStateDir)
		require.NoError(t, err)
		err = os.Remove(TestDataStoreFile + BeaconStoreName)
		require.NoError(t, err)
		store.Close()
	}()

	// entry 1
	pair1FinalizedSlot := uint64(4570816)
	pair1AttestedSlot := uint64(4570752)

	pair1AttestedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair1AttestedSlot))
	require.NoError(t, err)
	pair1FinalizedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair1FinalizedSlot))
	require.NoError(t, err)

	err = store.WriteEntry(pair1AttestedSlot, pair1FinalizedSlot, pair1AttestedState, pair1FinalizedState)
	require.NoError(t, err)

	// entry 2
	pair2FinalizedSlot := uint64(4571072)
	pair2AttestedSlot := uint64(4571136)

	pair2AttestedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair2AttestedSlot))
	require.NoError(t, err)
	pair2FinalizedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair2FinalizedSlot))
	require.NoError(t, err)

	err = store.WriteEntry(pair2AttestedSlot, pair2FinalizedSlot, pair2AttestedState, pair2FinalizedState)
	require.NoError(t, err)

	// entry 3
	pair3FinalizedSlot := uint64(4644864)
	pair3AttestedSlot := uint64(4644928)

	pair3AttestedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair3AttestedSlot))
	require.NoError(t, err)
	pair3FinalizedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair3FinalizedSlot))
	require.NoError(t, err)

	err = store.WriteEntry(pair3AttestedSlot, pair3FinalizedSlot, pair3AttestedState, pair3FinalizedState)
	require.NoError(t, err)

	_, err = store.GetBeaconStateData(pair1FinalizedSlot)
	require.NoError(t, err)
	_, err = store.GetBeaconStateData(pair1AttestedSlot)
	require.NoError(t, err)

	deleted, err := store.PruneOldStates()
	require.NoError(t, err)
	require.Equal(t, []uint64{pair1AttestedSlot, pair1FinalizedSlot}, deleted) // Check the oldest slots were deleted

	// Check the files were also deleted
	_, err = store.GetBeaconStateData(pair1FinalizedSlot)
	require.Error(t, err)
	_, err = store.GetBeaconStateData(pair1AttestedSlot)
	require.Error(t, err)
}

func TestFindBeaconStateWithinRange(t *testing.T) {
	_ = os.RemoveAll(TestDataStoreFile + BeaconStateDir)
	_ = os.Remove(TestDataStoreFile + BeaconStoreName)

	p := protocol.New(config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		ForkVersions: config.ForkVersions{
			Deneb:   0,
			Electra: 800000,
		},
	}, MaxRedundancy)
	store := New(TestDataStoreFile, 2, *p)
	err := store.Connect()
	require.NoError(t, err)
	defer func() {
		err := os.RemoveAll(TestDataStoreFile + BeaconStateDir)
		require.NoError(t, err)
		err = os.Remove(TestDataStoreFile + BeaconStoreName)
		require.NoError(t, err)
		store.Close()
	}()

	// entry 1
	pair1FinalizedSlot := uint64(4570816)
	pair1AttestedSlot := uint64(4570752)

	pair1AttestedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair1AttestedSlot))
	require.NoError(t, err)
	pair1FinalizedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair1FinalizedSlot))
	require.NoError(t, err)

	err = store.WriteEntry(pair1AttestedSlot, pair1FinalizedSlot, pair1AttestedState, pair1FinalizedState)
	require.NoError(t, err)

	// entry 2
	pair2FinalizedSlot := uint64(4571072)
	pair2AttestedSlot := uint64(4571136)

	pair2AttestedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair2AttestedSlot))
	require.NoError(t, err)
	pair2FinalizedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair2FinalizedSlot))
	require.NoError(t, err)

	err = store.WriteEntry(pair2AttestedSlot, pair2FinalizedSlot, pair2AttestedState, pair2FinalizedState)
	require.NoError(t, err)

	// entry 3
	pair3FinalizedSlot := uint64(4644864)
	pair3AttestedSlot := uint64(4644928)

	pair3AttestedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair3AttestedSlot))
	require.NoError(t, err)
	pair3FinalizedState, err := testutil.LoadFile(fmt.Sprintf("%d.ssz", pair3FinalizedSlot))
	require.NoError(t, err)

	err = store.WriteEntry(pair3AttestedSlot, pair3FinalizedSlot, pair3AttestedState, pair3FinalizedState)
	require.NoError(t, err)

	period568Start := uint64(568 * 256 * 32)
	beaconData, err := store.FindBeaconStateWithinRange(4644864, period568Start)
	require.NoError(t, err)

	require.Equal(t, pair3AttestedSlot, beaconData.AttestedSlot)
	require.Equal(t, pair3FinalizedSlot, beaconData.FinalizedSlot)

	period558Start := uint64(558 * 256 * 32)
	beaconData, err = store.FindBeaconStateWithinRange(4570003, period558Start)
	require.NoError(t, err)

	require.Equal(t, pair1AttestedSlot, beaconData.AttestedSlot)
	require.Equal(t, pair1FinalizedSlot, beaconData.FinalizedSlot)

	period559Start := uint64(559 * 256 * 32)
	beaconData, err = store.FindBeaconStateWithinRange(4570800, period559Start)
	require.NoError(t, err)

	require.Equal(t, int(pair1AttestedSlot), int(beaconData.AttestedSlot))
	require.Equal(t, int(pair1FinalizedSlot), int(beaconData.FinalizedSlot))
}
