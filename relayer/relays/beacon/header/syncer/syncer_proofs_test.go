package syncer

import (
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/stretchr/testify/require"
	"os"
	"testing"
)

/*
func TestHeaderProof_Mainnet(t *testing.T) {
	s := New("https://lodestar-goerli.chainsafe.io", 32, 512, 8192, config.Mainnet)

	finalizedUpdate, _, err := s.GetFinalizedUpdate(common.Hash{})
	require.NoError(t, err)

	fmt.Printf("finalized slot: %d\n", finalizedUpdate.FinalizedHeader.Slot)
	fmt.Printf("cached slot: %d\n", s.currentFinalizedHeader.slot)
	fmt.Printf("cached blockHash: %s\n", s.currentFinalizedHeader.blockHash)
	fmt.Printf("cached blockRootProofHash: %s\n", s.currentFinalizedHeader.blockRootProofHash)

	prevSlot := finalizedUpdate.FinalizedHeader.Slot - 1

	fmt.Printf("header slot: %d\n", prevSlot)

	_, err = s.GetNextHeaderUpdateBySlot(uint64(prevSlot))
	require.NoError(t, err)
}

func TestHeaderProof_Minimal(t *testing.T) {
	s := New("http://localhost:9596", 8, 8, 64, config.Minimal)

	finalizedUpdate, _, err := s.GetFinalizedUpdate(common.Hash{})
	require.NoError(t, err)

	fmt.Printf("cached slot: %d\n", s.currentFinalizedHeader.slot)
	fmt.Printf("cached blockHash: %s\n", s.currentFinalizedHeader.blockHash)
	fmt.Printf("cached blockRootProofHash: %s\n", s.currentFinalizedHeader.blockRootProofHash)

	prevSlot := finalizedUpdate.FinalizedHeader.Slot - 1

	fmt.Printf("header slot: %d\n", prevSlot)

	_, err = s.GetNextHeaderUpdateBySlot(uint64(prevSlot))
	require.NoError(t, err)
}*/

func TestMinimalHashTreeRoot(t *testing.T) {
	data, err := os.ReadFile("beacon_state.ssz")
	require.NoError(t, err)

	beaconState := &state.BeaconStateBellatrixMinimal{}

	err = beaconState.UnmarshalSSZ(data)
	require.NoError(t, err)

	fmt.Printf("slot: %d\n", beaconState.Slot)

	hh := ssz.Wrapper{}

	hh.PutUint64(beaconState.Slot)

	hh.Merkleize(0)

	fmt.Printf("slot hash: %s\n", common.BytesToHash(hh.Node().Hash()))

	stateTree, err := beaconState.GetTree()
	require.NoError(t, err)

	_, err = beaconState.HashTreeRoot()
	require.NoError(t, err)

	checkStateRoot := stateTree.Hash()

	//require.Equal(t, "0x8f354529fd13837c897fcbc2b510247f298c109adb510014066b6f8312af720b", common.BytesToHash(s1[:]).Hex())
	require.Equal(t, "0x8178dcfaab8b9947267d6032d514a4e417a7923570d3154fd041df6476306ccf", common.BytesToHash(checkStateRoot).Hex())
}
