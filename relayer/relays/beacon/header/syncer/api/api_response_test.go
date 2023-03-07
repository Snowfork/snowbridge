package api

import (
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/stretchr/testify/require"
	"io"
	"net/http"
	"os"
	"testing"
)

func TestExecutionPayloadToScale(t *testing.T) {
	data, err := os.ReadFile("beacon_block.ssz")
	require.NoError(t, err)

	beaconBlock := state.BeaconBlockBellatrixMainnet{}

	err = beaconBlock.UnmarshalSSZ(data)
	require.NoError(t, err)
}

func TestExecutionPayloadToScaleLocal(t *testing.T) {
	data, err := os.ReadFile("beacon_block_local.ssz")
	require.NoError(t, err)

	beaconBlock := state.BeaconBlockBellatrixMinimal{}

	err = beaconBlock.UnmarshalSSZ(data)
	require.NoError(t, err)
}

func TestDownloadBlock(t *testing.T) {
	api := NewBeaconClient("https://lodestar-goerli.chainsafe.io", config.Mainnet)

	beaconBlock, err := api.GetBeaconBlock(common.HexToHash("0x0be966e6710e7fe240fc93435a56f8a40f2c338b60cffdc67dd7f6f25d3016fc"))
	require.NoError(t, err)

	tree, err := beaconBlock.GetTree()
	require.NoError(t, err)

	hash := tree.Hash()

	fmt.Println("slot")
	fmt.Println(beaconBlock.GetBeaconSlot())
	fmt.Println(common.BytesToHash(hash))
}

func TestDownloadBlockMinimal(t *testing.T) {
	client := http.Client{}

	req, err := http.NewRequest(http.MethodGet, "http://localhost:9596/eth/v2/beacon/blocks/head", nil)
	require.NoError(t, err)

	req.Header.Add("Accept", "application/octet-stream")
	res, err := client.Do(req)
	require.NoError(t, err)
	require.Equal(t, 200, res.StatusCode)

	out, err := os.Create("beacon_block_local.ssz")
	require.NoError(t, err)

	defer out.Close()
	io.Copy(out, res.Body)
}
