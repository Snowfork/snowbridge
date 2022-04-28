package beacon

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"golang.org/x/sync/errgroup"
)

type InitialSync struct {
	Header                     interface{}
	CurrentSyncCommittee       interface{}
	CurrentSyncCommitteeBranch interface{}
	ValidatorsRoot             interface{}
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

func (wr *ParachainWriter) WritePayload(ctx context.Context, payload *ParachainPayload, eg *errgroup.Group) error {
	return wr.write(ctx)
}

// Write submits a transaction to the chain
func (wr *ParachainWriter) write(ctx context.Context) error {
	meta, err := wr.conn.API().RPC.State.GetMetadataLatest()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "EthereumBeaconLightClient.simple_test")
	if err != nil {
		return err
	}

	ext := types.NewExtrinsic(c)

	latestHash, err := wr.conn.API().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return err
	}

	_, err = wr.conn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return err
	}

	genesisHash, err := wr.conn.API().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}

	rv, err := wr.conn.API().RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return err
	}

	era := types.ExtrinsicEra{IsImmortalEra: false}

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
		BlockHash:          genesisHash,
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

	logrus.WithFields(logrus.Fields{
		"nonce": nonce,
	}).Info("Submitting transaction")

	logrus.WithFields(logrus.Fields{
		"signature_options": o,
	}).Info("Signature options")

	logrus.WithFields(logrus.Fields{
		"signature_options": o.Nonce.Int64(),
	}).Info("Nonce")

	logrus.WithFields(logrus.Fields{
		"signature_options": o.Tip.Int64(),
	}).Info("Tip")

	_, err = wr.conn.API().RPC.Author.SubmitExtrinsic(ext)
	//err = wr.pool.WaitForSubmitAndWatch(ctx, &extI, onFinalized)
	if err != nil {
		logrus.WithError(err).WithField("nonce", wr.nonce).Debug("Failed to submit extrinsic")
		return err
	}

	wr.nonce = wr.nonce + 1

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
