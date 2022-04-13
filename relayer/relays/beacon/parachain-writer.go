package beacon

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
)

type InitialSync struct {
	Header                     syncer.Header
	CurrentSyncCommittee       syncer.CurrentSyncCommittee
	CurrentSyncCommitteeBranch []string
	Genesis                    syncer.Genesis
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

func (wr *ParachainWriter) WritePayload(ctx context.Context, payload *ParachainPayload) error {
	call, err := wr.makeInitialSyncCall(payload.InitialSync)
	if err != nil {
		return err
	}

	onFinalized := func(_ types.Hash) error {
		// Confirm that the header import was successful
		headerHash := payload.InitialSync.Header.BodyRoot
		imported, err := wr.queryImportedHeaderExists(headerHash)
		if err != nil {
			return err
		}
		if !imported {
			return fmt.Errorf("Header import failed for header %s", headerHash.Hex())
		}
		return nil
	}

	return wr.write(ctx, call, onFinalized)
}

// Write submits a transaction to the chain
func (wr *ParachainWriter) write(
	ctx context.Context,
	c types.Call,
	onFinalized parachain.OnFinalized,
) error {
	ext := types.NewExtrinsic(c)

	latestHash, err := wr.conn.API().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return err
	}

	latestBlock, err := wr.conn.API().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return err
	}

	era := parachain.NewMortalEra(uint64(latestBlock.Block.Header.Number))

	rv, err := wr.conn.API().RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return err
	}

	o := types.SignatureOptions{
		BlockHash:          latestHash,
		Era:                era,
		GenesisHash:        wr.genesisHash,
		Nonce:              types.NewUCompactFromUInt(uint64(wr.nonce)),
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
		"nonce": wr.nonce,
	}).Info("Submitting transaction")
	err = wr.pool.WaitForSubmitAndWatch(ctx, &extI, onFinalized)
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

	return types.NewCall(wr.conn.Metadata(), "EthereumLightClient.intial_sync", initialSync.Header, initialSync.CurrentSyncCommittee, initialSync.CurrentSyncCommitteeBranch, initialSync.Genesis)
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
