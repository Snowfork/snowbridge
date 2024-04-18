package cache

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/require"
	"testing"
)

func TestCalculateClosestCheckpointSlot(t *testing.T) {
	b := New(32, 256)

	b.AddCheckPoint(common.HexToHash("0xfa767e1fb1280799fd406bd7905d3bef62d498211183548f9ebb7a1d16edce4c"), nil, 16)
	b.AddCheckPoint(common.HexToHash("0xe5509a901249bcb4800b644ebb3c666074848ea02d0e85427fff29fe2ec354ec"), nil, 64)
	b.AddCheckPoint(common.HexToHash("0xecdf3404d4909e5ef6315566ae0cca2c20bf2e6ec6c18f4d26fc7913d9eaa592"), nil, 128)

	slot, err := b.calculateClosestCheckpointSlot(17)
	require.NoError(t, err)
	// 128 because 64 would be overwritten
	require.Equal(t, uint64(128), slot)
}

func TestCalculateClosestCheckpointSlot_WithoutCheckpointIncludingSlot(t *testing.T) {
	b := New(32, 256)

	b.AddCheckPoint(common.HexToHash("0xe5509a901249bcb4800b644ebb3c666074848ea02d0e85427fff29fe2ec354ec"), nil, 8250)
	b.AddCheckPoint(common.HexToHash("0xecdf3404d4909e5ef6315566ae0cca2c20bf2e6ec6c18f4d26fc7913d9eaa592"), nil, 8500)

	_, err := b.calculateClosestCheckpointSlot(32)
	require.Error(t, err)
}

func TestCalculateClosestCheckpointSlot_WithoutCheckpointIncludingSlotTooLarge(t *testing.T) {
	b := New(32, 256)

	b.AddCheckPoint(common.HexToHash("0xe5509a901249bcb4800b644ebb3c666074848ea02d0e85427fff29fe2ec354ec"), nil, 72)
	b.AddCheckPoint(common.HexToHash("0xecdf3404d4909e5ef6315566ae0cca2c20bf2e6ec6c18f4d26fc7913d9eaa592"), nil, 144)

	_, err := b.calculateClosestCheckpointSlot(145)
	require.Error(t, err)
}

func TestCalculateClosestCheckpointSlot_WithCheckpointMatchingSlot(t *testing.T) {
	b := New(32, 256)

	b.AddCheckPoint(common.HexToHash("0xe5509a901249bcb4800b644ebb3c666074848ea02d0e85427fff29fe2ec354ec"), nil, 72)
	b.AddCheckPoint(common.HexToHash("0xecdf3404d4909e5ef6315566ae0cca2c20bf2e6ec6c18f4d26fc7913d9eaa592"), nil, 144)

	slot, err := b.calculateClosestCheckpointSlot(144)
	require.NoError(t, err)
	require.Equal(t, uint64(144), slot)
}

func TestCalculateClosestCheckpointSlot_WithMoreThanOneCheckpoint(t *testing.T) {
	b := New(32, 256)

	b.AddCheckPoint(common.HexToHash("0xe5509a901249bcb4800b644ebb3c666074848ea02d0e85427fff29fe2ec354ec"), nil, 32)
	b.AddCheckPoint(common.HexToHash("0xecdf3404d4909e5ef6315566ae0cca2c20bf2e6ec6c18f4d26fc7913d9eaa592"), nil, 16)

	slot, err := b.calculateClosestCheckpointSlot(2)
	require.NoError(t, err)
	require.Equal(t, uint64(16), slot) // taking the first matching checkpoint is fine
}

func TestAddSlot(t *testing.T) {
	b := New(32, 256)

	b.addSlot(5)
	require.Equal(t, []uint64{5}, b.Finalized.Checkpoints.Slots)

	b.addSlot(10)
	require.Equal(t, []uint64{5, 10}, b.Finalized.Checkpoints.Slots)

	b.addSlot(10) // test duplicate slot add
	require.Equal(t, []uint64{5, 10}, b.Finalized.Checkpoints.Slots)

	b.addSlot(6) // test duplicate slot add
	require.Equal(t, []uint64{5, 6, 10}, b.Finalized.Checkpoints.Slots)
}

func TestAddCheckpointOverwritesCorrectFinalizedCheckpoints(t *testing.T) {
	b := New(32, 256)

	b.AddCheckPoint(common.HexToHash("0xe5509a901249bcb4800b644ebb3c666074848ea02d0e85427fff29fe2ec354ec"), nil, 16)
	require.Equal(t, []uint64{16}, b.Finalized.Checkpoints.Slots)

	b.AddCheckPoint(common.HexToHash("0xecdf3404d4909e5ef6315566ae0cca2c20bf2e6ec6c18f4d26fc7913d9eaa592"), nil, 32)
	require.Equal(t, []uint64{16, 32}, b.Finalized.Checkpoints.Slots)

	b.AddCheckPoint(common.HexToHash("0xf24b5b5f67006c67890d6b0e519695f1699f0b1916670aaaa41f9fe00cb55751"), nil, 64)
	require.Equal(t, []uint64{16, 64}, b.Finalized.Checkpoints.Slots)

	b.AddCheckPoint(common.HexToHash("0xba6c73bcbe2b868800c1ae9785c5e6b17724ce557c14837965ecacf49ec2d502"), nil, 96)
	require.Equal(t, []uint64{16, 96}, b.Finalized.Checkpoints.Slots)

	b.AddCheckPoint(common.HexToHash("0xfae5503eb892cf097ee29eb005eb4f7513cf0697c6fcf3502a83bb8e4b4ea16d"), nil, 8288)
	require.Equal(t, []uint64{16, 96, 8288}, b.Finalized.Checkpoints.Slots)

	b.AddCheckPoint(common.HexToHash("0x20c56947d004f6e3a1433e4063def46c375602c51379709735361465d0cb7c57"), nil, 8352)
	require.Equal(t, []uint64{16, 96, 8288, 8352}, b.Finalized.Checkpoints.Slots)

	b.AddCheckPoint(common.HexToHash("0xfe224a0c7c0502b5be4418d0bec2e7108280b790e8b98bec412e90397c1c3ed8"), nil, 16544)
	require.Equal(t, []uint64{16, 96, 8288, 8352, 16544}, b.Finalized.Checkpoints.Slots)
}

func TestOverwriteSlot(t *testing.T) {
	b := New(32, 256)

	b.addSlot(5)
	require.Equal(t, []uint64{5}, b.Finalized.Checkpoints.Slots)

	b.addSlot(10)
	require.Equal(t, []uint64{5, 10}, b.Finalized.Checkpoints.Slots)

	overwrittenSlot := b.overwriteSlot(20)
	require.Equal(t, []uint64{5, 20}, b.Finalized.Checkpoints.Slots)
	require.Equal(t, uint64(10), overwrittenSlot)

	b.addSlot(26)
	require.Equal(t, []uint64{5, 20, 26}, b.Finalized.Checkpoints.Slots)

	overwrittenSlot = b.overwriteSlot(30)
	require.Equal(t, []uint64{5, 20, 30}, b.Finalized.Checkpoints.Slots)
	require.Equal(t, uint64(26), overwrittenSlot)
}

func TestAddPruneOldCheckpoints(t *testing.T) {
	b := New(32, 256)
	slotsPerHistoricalPeriod := 8192

	for i := 1; i <= FinalizedCheckpointsLimit+20; i++ {
		b.AddCheckPoint(common.HexToHash("0xe5509a901249bcb4800b644ebb3c666074848ea02d0e85427fff29fe2ec354ec"), nil, uint64(i*slotsPerHistoricalPeriod))
	}

	require.Equal(t, FinalizedCheckpointsLimit, len(b.Finalized.Checkpoints.Slots))

	for _, checkpoint := range b.Finalized.Checkpoints.Proofs {
		// check that each slot is within the expected range
		require.Greater(t, checkpoint.Slot, uint64(20*slotsPerHistoricalPeriod))
		require.LessOrEqual(t, checkpoint.Slot, uint64((FinalizedCheckpointsLimit+20)*slotsPerHistoricalPeriod))
	}
}
