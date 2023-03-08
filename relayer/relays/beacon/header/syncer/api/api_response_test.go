package api

import (
	"fmt"
	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/util"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/stretchr/testify/require"
	"io"
	"math/big"
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

	execHtr, err := beaconBlock.GetExecutionPayload().HashTreeRoot()
	require.Equal(t, "0x6282d3f6de498b2ce0764adf601498d5695190396abe804e95f15de8b8e2810e", common.BytesToHash(execHtr[:]).Hex())

	fmt.Println(common.BytesToHash(execHtr[:]))

	beaconBlockRoot, err := api.GetBeaconBlockRoot(beaconBlock.GetBeaconSlot())
	require.NoError(t, err)

	tree, err := beaconBlock.GetTree()
	require.NoError(t, err)

	hash := tree.Hash()

	fmt.Println("slot")
	fmt.Println(beaconBlock.GetBeaconSlot())

	proof, err := tree.Prove(201)
	require.NoError(t, err)

	fmt.Println("leaf:" + common.BytesToHash(proof.Leaf[:]).Hex())
	for _, proofItem := range proof.Hashes {
		fmt.Println(common.BytesToHash(proofItem[:]))
	}

	root, err := beaconBlock.GetExecutionPayload().HashTreeRoot()
	require.NoError(t, err)

	require.Equal(t, common.BytesToHash(root[:]), common.BytesToHash(proof.Leaf[:]))

	ok, err := ssz.VerifyProof(hash, proof)
	require.NoError(t, err)
	require.True(t, ok)

	require.Equal(t, beaconBlockRoot, common.BytesToHash(hash))
}

func TestBaseFeePerGas(t *testing.T) {
	strValue := "161912342325"

	n := new(big.Int)
	n, ok := n.SetString(strValue, 10)
	require.True(t, ok)

	baseFeePerGas := n.Bytes()

	// convert to little endian, ew
	for i := 0; i < len(baseFeePerGas)/2; i++ {
		baseFeePerGas[i], baseFeePerGas[len(baseFeePerGas)-i-1] = baseFeePerGas[len(baseFeePerGas)-i-1], baseFeePerGas[i]
	}
	var baseFeePerGasBytes [32]byte
	copy(baseFeePerGasBytes[:], baseFeePerGas)

	fmt.Println(baseFeePerGas)

	for i := 0; i < len(baseFeePerGas)/2; i++ {
		baseFeePerGas[i], baseFeePerGas[len(baseFeePerGas)-i-1] = baseFeePerGas[len(baseFeePerGas)-i-1], baseFeePerGas[i]
	}

	s := new(big.Int)
	s.SetBytes(baseFeePerGas)
	fmt.Println(s.String())
}

func TestDownloadBlock_ExecutionHeaderPayload(t *testing.T) {
	api := NewBeaconClient("https://lodestar-goerli.chainsafe.io", config.Mainnet)

	beaconBlock, err := api.GetBeaconBlock(common.HexToHash("0x0be966e6710e7fe240fc93435a56f8a40f2c338b60cffdc67dd7f6f25d3016fc"))
	require.NoError(t, err)

	exec := beaconBlock.GetExecutionPayload()
	execHtr, err := exec.HashTreeRoot()

	fmt.Println("parent root:" + common.BytesToHash(exec.ParentHash[:]).Hex())
	fmt.Println("fee recipient:" + util.BytesToHexString(exec.FeeRecipient[:]))
	fmt.Println("state root:" + common.BytesToHash(exec.StateRoot[:]).Hex())
	fmt.Println("receipts root:" + common.BytesToHash(exec.ReceiptsRoot[:]).Hex())
	fmt.Println("logs bloom:" + util.BytesToHexString(exec.LogsBloom[:]))
	fmt.Println("prev_randao:" + util.BytesToHexString(exec.PrevRandao[:]))
	fmt.Printf("block_number: %d\n", exec.BlockNumber)
	fmt.Printf("gas_limit: %d\n", exec.GasLimit)
	fmt.Printf("gas_used: %d\n", exec.GasUsed)
	fmt.Printf("timestamp: %d\n", exec.Timestamp)
	fmt.Println("extra_data:" + util.BytesToHexString(exec.ExtraData[:]))

	require.Equal(t, "0x6282d3f6de498b2ce0764adf601498d5695190396abe804e95f15de8b8e2810e", common.BytesToHash(execHtr[:]).Hex())
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
