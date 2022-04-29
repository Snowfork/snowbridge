package beacon

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
	"golang.org/x/sync/errgroup"
)

type BeaconHeaderScale struct {
	Slot          types.U64
	ProposerIndex types.U64
	ParentRoot    types.H256
	StateRoot     types.H256
	BodyRoot      types.H256
}

type CurrentSyncCommitteeScale struct {
	Pubkeys         [][48]byte
	AggregatePubkey [48]byte
}

type InitialSync struct {
	Header                     BeaconHeaderScale
	CurrentSyncCommittee       CurrentSyncCommitteeScale
	CurrentSyncCommitteeBranch []types.H256
	ValidatorsRoot             types.H256
}

type ParachainPayload struct {
	InitialSync *InitialSync
	Messages    []*chain.EthereumOutboundMessage
}

type ParachainWriter struct {
	conn        *parachain.Connection
	nonce       uint32
	pool        *parachain.ExtrinsicPool
	genesisHash types.Hash
}

func NewParachainWriter(
	conn *parachain.Connection,
) *ParachainWriter {
	return &ParachainWriter{
		conn: conn,
	}
}

func (wr *ParachainWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	nonce, err := wr.queryAccountNonce()
	if err != nil {
		return err
	}
	wr.nonce = nonce

	genesisHash, err := wr.conn.API().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}
	wr.genesisHash = genesisHash

	wr.pool = parachain.NewExtrinsicPool(eg, wr.conn)

	return nil
}

func (wr *ParachainWriter) queryAccountNonce() (uint32, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "System", "Account", wr.conn.Keypair().PublicKey, nil)
	if err != nil {
		return 0, err
	}

	var accountInfo types.AccountInfo
	ok, err := wr.conn.API().RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil {
		return 0, err
	}
	if !ok {
		return 0, fmt.Errorf("no account info found for %s", wr.conn.Keypair().URI)
	}

	return uint32(accountInfo.Nonce), nil
}

func (wr *ParachainWriter) WritePayload(ctx context.Context, initialSync syncer.LightClientSnapshot, eg *errgroup.Group) error {
	return wr.write(ctx, initialSync)
}

// Write submits a transaction to the chain
func (wr *ParachainWriter) write(ctx context.Context, snapshot syncer.LightClientSnapshot) error {
	meta, err := wr.conn.API().RPC.State.GetMetadataLatest()
	if err != nil {
		return err
	}

	is := InitialSync{
		Header: BeaconHeaderScale{
			Slot:          types.NewU64(snapshot.Header.Slot),
			ProposerIndex: types.NewU64(snapshot.Header.ProposerIndex),
			ParentRoot:    types.NewH256(snapshot.Header.ParentRoot.Bytes()),
			StateRoot:     types.NewH256(snapshot.Header.StateRoot.Bytes()),
			BodyRoot:      types.NewH256(snapshot.Header.BodyRoot.Bytes()),
		},
		ValidatorsRoot: types.NewH256([]byte(snapshot.ValidatorsRoot)),
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

	syncCommitteeBranch := []types.H256{}

	for _, branch := range snapshot.CurrentSyncCommitteeBranch {
		syncCommitteeBranch = append(syncCommitteeBranch, types.NewH256([]byte(branch)))
	}

	is.CurrentSyncCommitteeBranch = syncCommitteeBranch

	c, err := types.NewCall(meta, "EthereumBeaconLightClient.initial_sync", is)
	if err != nil {
		return err
	}

	latestHash, err := wr.conn.API().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return err
	}

	latestBlock, err := wr.conn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return err
	}

	ext := types.NewExtrinsic(c)
	era := parachain.NewMortalEra(uint64(latestBlock.Block.Header.Number))

	genesisHash, err := wr.conn.API().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}

	rv, err := wr.conn.API().RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return err
	}

	key, err := types.CreateStorageKey(meta, "System", "Account", wr.conn.Keypair().PublicKey)
	if err != nil {
		return err
	}

	var accountInfo types.AccountInfo
	_, err = wr.conn.API().RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil {
		return err
	}

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

	err = extI.Sign(*wr.conn.Keypair(), o)
	if err != nil {
		return err
	}

	_, err = wr.conn.API().RPC.Author.SubmitAndWatchExtrinsic(extI)
	if err != nil {
		return err
	}

	return nil
}

func (wr *ParachainWriter) makeInitialSyncCall(initialSync *InitialSync) (types.Call, error) {
	if initialSync == (*InitialSync)(nil) {
		return types.Call{}, fmt.Errorf("Initial sync is nil")
	}

	//return types.NewCall(wr.conn.Metadata(), "EthereumBeaconLightClient.initial_sync", initialSync.Header, initialSync.CurrentSyncCommittee, initialSync.CurrentSyncCommitteeBranch, initialSync.Genesis)
	return types.NewCall(wr.conn.Metadata(), "EthereumBeaconLightClient.simple_test")
}

func (wr *ParachainWriter) queryImportedHeaderExists(hash common.Hash) (bool, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumBeaconLightClient", "FinalizedHeaders", hash[:], nil)
	if err != nil {
		return false, err
	}

	var headerOption types.OptionBytes
	ok, err := wr.conn.API().RPC.State.GetStorageLatest(key, &headerOption)
	if err != nil {
		return false, err
	}
	if !ok {
		return false, fmt.Errorf("Storage query did not find header for hash %s", hash.Hex())
	}

	return headerOption.IsSome(), nil
}
