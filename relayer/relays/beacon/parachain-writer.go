package beacon

import (
	"context"
	"fmt"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
)

type BeaconHeader struct {
	HeaderData interface{}
	ProofData  interface{}
}

type ParachainPayload struct {
	Header   *chain.Header
	Messages []*chain.EthereumOutboundMessage
}

type ParachainWriter struct {
	conn        *parachain.Connection
	payloads    <-chan ParachainPayload
	nonce       uint32
	pool        *parachain.ExtrinsicPool
	genesisHash types.Hash
}

func NewParachainWriter(
	conn *parachain.Connection,
	payloads <-chan ParachainPayload,
) *ParachainWriter {
	return &ParachainWriter{
		conn:     conn,
		payloads: payloads,
	}
}

func (wr *ParachainWriter) WritePayload(ctx context.Context, payload *ParachainPayload) error {
	call, err := wr.makeInitialSyncCall(payload.Header)
	if err != nil {
		return err
	}

	onFinalized := func(_ types.Hash) error {
		// Confirm that the header import was successful
		header := payload.Header.HeaderData.(ethereum.Header)
		hash := header.ID().Hash
		imported, err := wr.queryImportedHeaderExists(hash)
		if err != nil {
			return err
		}
		if !imported {
			return fmt.Errorf("Header import failed for header %s", hash.Hex())
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

func (wr *ParachainWriter) makeInitialSyncCall(header *chain.Header) (types.Call, error) {
	if header == (*chain.Header)(nil) {
		return types.Call{}, fmt.Errorf("Header is nil")
	}

	return types.NewCall(wr.conn.Metadata(), "EthereumLightClient.import_header", header.HeaderData, header.ProofData)
}

func (wr *ParachainWriter) queryImportedHeaderExists(hash types.H256) (bool, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumLightClient", "FinalizedHeaders", hash[:], nil)
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
