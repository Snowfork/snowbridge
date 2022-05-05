package beacon

import (
	"testing"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestChain_SubmitExtrinsic_InitialSync(t *testing.T) {
	syncer := syncer.New("https://lodestar-kiln.chainsafe.io")

	initialSync, err := syncer.InitialSync("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
	require.NoError(t, err)

	from, ok := signature.LoadKeyringPairFromEnv()
	if !ok {
		from = signature.TestKeyringPairAlice
	}

	api, err := gsrpc.NewSubstrateAPI("ws://127.0.0.1:11144")
	require.NoError(t, err)

	meta, err := api.RPC.State.GetMetadataLatest()
	require.NoError(t, err)

	c, err := types.NewCall(meta, "EthereumBeaconLightClient.initial_sync", initialSync)
	assert.NoError(t, err)

	latestHash, err := api.RPC.Chain.GetFinalizedHead()
	assert.NoError(t, err)

	latestBlock, err := api.RPC.Chain.GetBlock(latestHash)
	assert.NoError(t, err)

	ext := types.NewExtrinsic(c)
	era := parachain.NewMortalEra(uint64(latestBlock.Block.Header.Number))

	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	assert.NoError(t, err)

	rv, err := api.RPC.State.GetRuntimeVersionLatest()
	assert.NoError(t, err)

	key, err := types.CreateStorageKey(meta, "System", "Account", from.PublicKey)
	assert.NoError(t, err)

	var accountInfo types.AccountInfo
	_, err = api.RPC.State.GetStorageLatest(key, &accountInfo)
	assert.NoError(t, err)

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		BlockHash:          latestHash,
		Era:                era,
		GenesisHash:        genesisHash,
		Nonce:              types.NewUCompactFromUInt(uint64(nonce)),
		SpecVersion:        rv.SpecVersion,
		Tip:                types.NewUCompactFromUInt(0),
		TransactionVersion: rv.TransactionVersion,
	}

	extI := ext

	err = extI.Sign(from, o)
	assert.NoError(t, err)

	_, err = api.RPC.Author.SubmitExtrinsic(extI)
	assert.NoError(t, err)
}
