package beacon

import (
	"testing"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/config"
	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
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

func TestChain_SubmitExtrinsic_InitialSync(t *testing.T) {
	syncer := syncer.New("https://lodestar-kiln.chainsafe.io", "https://lodestar-kiln.chainsafe.io")

	snapshot, err := syncer.InitialSync("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
	assert.NoError(t, err)

	from, ok := signature.LoadKeyringPairFromEnv()
	if !ok {
		from = signature.TestKeyringPairAlice
	}

	api, err := gsrpc.NewSubstrateAPI("ws://127.0.0.1:11144")
	assert.NoError(t, err)

	meta, err := api.RPC.State.GetMetadataLatest()
	assert.NoError(t, err)

	type InitialSyncLocal struct {
		Header               BeaconHeaderScale
		CurrentSyncCommittee CurrentSyncCommitteeScale
	}

	is := InitialSyncLocal{
		Header: BeaconHeaderScale{
			Slot:          types.NewU64(snapshot.Header.Slot),
			ProposerIndex: types.NewU64(snapshot.Header.ProposerIndex),
			ParentRoot:    types.NewH256(snapshot.Header.ParentRoot.Bytes()),
			StateRoot:     types.NewH256(snapshot.Header.StateRoot.Bytes()),
			BodyRoot:      types.NewH256(snapshot.Header.BodyRoot.Bytes()),
		},
	}

	var syncCommitteePubkeysScale = make([][48]byte, 512)

	for _, pubkey := range snapshot.CurrentSyncCommittee.Pubkeys {
		var pubkeyBytes [48]byte
		copy(pubkeyBytes[:], pubkey)
		syncCommitteePubkeysScale = append(syncCommitteePubkeysScale, pubkeyBytes)
	}

	var aggPubkey [48]byte
	copy(aggPubkey[:], snapshot.CurrentSyncCommittee.AggregatePubkeys)

	is.CurrentSyncCommittee = CurrentSyncCommitteeScale{
		Pubkeys:         syncCommitteePubkeysScale,
		AggregatePubkey: aggPubkey,
	}

	c, err := types.NewCall(meta, "EthereumBeaconLightClient.initial_sync", is)
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
