package beacon

import (
	"testing"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/config"
	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestChain_SubmitExtrinsic(t *testing.T) {
	from, ok := signature.LoadKeyringPairFromEnv()
	if !ok {
		from = signature.TestKeyringPairAlice
	}

	api, err := gsrpc.NewSubstrateAPI(config.Default().RPCURL)
	assert.NoError(t, err)

	meta, err := api.RPC.State.GetMetadataLatest()
	assert.NoError(t, err)

	//bob, err := types.NewMultiAddressFromHexAccountID("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")
	//assert.NoError(t, err)

	//c, err := types.NewCall(meta, "System.remark")
	c, err := types.NewCall(meta, "System.remark", []byte("Clara is testing"))
	assert.NoError(t, err)

	ext := types.NewExtrinsic(c)
	era := types.ExtrinsicEra{IsImmortalEra: false}

	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	assert.NoError(t, err)

	rv, err := api.RPC.State.GetRuntimeVersionLatest()
	assert.NoError(t, err)

	key, err := types.CreateStorageKey(meta, "System", "Account", from.PublicKey)
	assert.NoError(t, err)

	var accountInfo types.AccountInfo
	ok, err = api.RPC.State.GetStorageLatest(key, &accountInfo)
	assert.NoError(t, err)
	assert.True(t, ok)

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		// BlockHash:   blockHash,
		BlockHash:          genesisHash, // BlockHash needs to == GenesisHash if era is immortal. // TODO: add an error?
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

	txn, err := api.RPC.Author.SubmitExtrinsic(extI)
	assert.NoError(t, err)

	assert.NotEmpty(t, txn)
}

func TestChain_SubmitExtrinsic_SimpleTest(t *testing.T) {
	from, ok := signature.LoadKeyringPairFromEnv()
	if !ok {
		from = signature.TestKeyringPairAlice
	}

	api, err := gsrpc.NewSubstrateAPI("ws://127.0.0.1:11144")
	assert.NoError(t, err)

	meta, err := api.RPC.State.GetMetadataLatest()
	assert.NoError(t, err)

	//bob, err := types.NewMultiAddressFromHexAccountID("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")
	//assert.NoError(t, err)

	//c, err := types.NewCall(meta, "System.remark")
	c, err := types.NewCall(meta, "EthereumBeaconLightClient.simple_test")
	require.NoError(t, err)

	ext := types.NewExtrinsic(c)
	era := types.ExtrinsicEra{IsImmortalEra: false}

	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	assert.NoError(t, err)

	rv, err := api.RPC.State.GetRuntimeVersionLatest()
	assert.NoError(t, err)

	//latestHash, err := api.RPC.Chain.GetFinalizedHead()
	_, err = api.RPC.Chain.GetFinalizedHead()
	assert.NoError(t, err)

	key, err := types.CreateStorageKey(meta, "System", "Account", from.PublicKey)
	assert.NoError(t, err)

	var accountInfo types.AccountInfo
	ok, err = api.RPC.State.GetStorageLatest(key, &accountInfo)
	assert.NoError(t, err)
	assert.True(t, ok)

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		//BlockHash: latestHash,
		BlockHash:          genesisHash, // BlockHash needs to == GenesisHash if era is immortal. // TODO: add an error?
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

func TestChain_SubmitExtrinsic_SimpleTest_With_Param(t *testing.T) {
	from, ok := signature.LoadKeyringPairFromEnv()
	if !ok {
		from = signature.TestKeyringPairAlice
	}

	api, err := gsrpc.NewSubstrateAPI("ws://127.0.0.1:11144")
	assert.NoError(t, err)

	meta, err := api.RPC.State.GetMetadataLatest()
	assert.NoError(t, err)

	//bob, err := types.NewMultiAddressFromHexAccountID("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")
	//assert.NoError(t, err)

	//c, err := types.NewCall(meta, "System.remark")
	//c, err := types.NewCall(meta, "EthereumBeaconLightClient.simple_test_with_param", types.NewH256([]byte("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")))
	c, err := types.NewCall(meta, "EthereumBeaconLightClient.simple_test_with_param", types.NewH256([]byte("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4704f26a48")))
	require.NoError(t, err)

	ext := types.NewExtrinsic(c)
	era := types.ExtrinsicEra{IsImmortalEra: false}

	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	assert.NoError(t, err)

	rv, err := api.RPC.State.GetRuntimeVersionLatest()
	assert.NoError(t, err)

	//latestHash, err := api.RPC.Chain.GetFinalizedHead()
	_, err = api.RPC.Chain.GetFinalizedHead()
	assert.NoError(t, err)

	key, err := types.CreateStorageKey(meta, "System", "Account", from.PublicKey)
	assert.NoError(t, err)

	var accountInfo types.AccountInfo
	ok, err = api.RPC.State.GetStorageLatest(key, &accountInfo)
	assert.NoError(t, err)
	assert.True(t, ok)

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		//BlockHash: latestHash,
		BlockHash:          genesisHash, // BlockHash needs to == GenesisHash if era is immortal. // TODO: add an error?
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

func TestChain_SubmitExtrinsic_SimpleTest_With_Struct(t *testing.T) {
	from, ok := signature.LoadKeyringPairFromEnv()
	if !ok {
		from = signature.TestKeyringPairAlice
	}

	api, err := gsrpc.NewSubstrateAPI("ws://127.0.0.1:11144")
	assert.NoError(t, err)

	meta, err := api.RPC.State.GetMetadataLatest()
	assert.NoError(t, err)

	type SigningData struct {
		ObjectRoot types.H256
		Domain     types.H256
	}	

	sd := SigningData{
		ObjectRoot: types.NewH256([]byte("0x9eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4704f26a48")),
		Domain:     types.NewH256([]byte("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4704f26a40")),
	}

	c, err := types.NewCall(meta, "EthereumBeaconLightClient.simple_test_with_struct", sd)
	require.NoError(t, err)

	ext := types.NewExtrinsic(c)
	era := types.ExtrinsicEra{IsImmortalEra: false}

	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	assert.NoError(t, err)

	rv, err := api.RPC.State.GetRuntimeVersionLatest()
	assert.NoError(t, err)

	//latestHash, err := api.RPC.Chain.GetFinalizedHead()
	_, err = api.RPC.Chain.GetFinalizedHead()
	assert.NoError(t, err)

	key, err := types.CreateStorageKey(meta, "System", "Account", from.PublicKey)
	assert.NoError(t, err)

	var accountInfo types.AccountInfo
	ok, err = api.RPC.State.GetStorageLatest(key, &accountInfo)
	assert.NoError(t, err)
	assert.True(t, ok)

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		//BlockHash: latestHash,
		BlockHash:          genesisHash, // BlockHash needs to == GenesisHash if era is immortal. // TODO: add an error?
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

func TestChain_SubmitExtrinsic_SimpleTest_With_Struct_With_ERA(t *testing.T) {
	from, ok := signature.LoadKeyringPairFromEnv()
	if !ok {
		from = signature.TestKeyringPairAlice
	}

	api, err := gsrpc.NewSubstrateAPI("ws://127.0.0.1:11144")
	assert.NoError(t, err)

	meta, err := api.RPC.State.GetMetadataLatest()
	assert.NoError(t, err)

	sd := SigningData{
		ObjectRoot: types.NewH256([]byte("0x9eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4704f26a48")),
		Domain:     types.NewH256([]byte("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4704f26a40")),
	}

	c, err := types.NewCall(meta, "EthereumBeaconLightClient.simple_test_with_struct", sd)
	require.NoError(t, err)

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
	ok, err = api.RPC.State.GetStorageLatest(key, &accountInfo)
	assert.NoError(t, err)
	assert.True(t, ok)

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		BlockHash: latestHash,
		//BlockHash:          genesisHash, // BlockHash needs to == GenesisHash if era is immortal. // TODO: add an error?
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
