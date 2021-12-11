package ethereum

import (
	"context"
	"errors"
	"fmt"

	"github.com/sirupsen/logrus"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"

	log "github.com/sirupsen/logrus"
)

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

	eg.Go(func() error {
		err := wr.writeLoop(ctx)
		log.WithField("reason", err).Info("Shutting down parachain writer")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})

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

func (wr *ParachainWriter) writeLoop(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case payload, ok := <-wr.payloads:
			if !ok {
				return nil
			}

			header := payload.Header.HeaderData.(ethereum.Header)
			err := wr.WritePayload(ctx, &payload)
			if err != nil {
				log.WithError(err).WithFields(logrus.Fields{
					"blockNumber":  header.Fields.Number,
					"messageCount": len(payload.Messages),
				}).Error("Failure submitting header and messages to Substrate")
				return err
			}

			log.WithFields(logrus.Fields{
				"blockNumber":  header.Fields.Number,
				"messageCount": len(payload.Messages),
			}).Info("Submitted transaction to Substrate")
		}
	}
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

	log.WithFields(logrus.Fields{
		"nonce": wr.nonce,
	}).Info("Submitting transaction")
	err = wr.pool.WaitForSubmitAndWatch(ctx, &extI, onFinalized)
	if err != nil {
		log.WithError(err).WithField("nonce", wr.nonce).Debug("Failed to submit extrinsic")
		return err
	}

	wr.nonce = wr.nonce + 1

	return nil
}

func (wr *ParachainWriter) WritePayload(ctx context.Context, payload *ParachainPayload) error {
	var calls []types.Call
	call, err := wr.makeHeaderImportCall(payload.Header)
	if err != nil {
		return err
	}
	calls = append(calls, call)

	for _, msg := range payload.Messages {
		call, err := wr.makeMessageSubmitCall(msg)
		if err != nil {
			return err
		}
		calls = append(calls, call)
	}

	call, err = types.NewCall(wr.conn.Metadata(), "Utility.batch_all", calls)
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

func (wr *ParachainWriter) makeMessageSubmitCall(msg *chain.EthereumOutboundMessage) (types.Call, error) {
	if msg == (*chain.EthereumOutboundMessage)(nil) {
		return types.Call{}, fmt.Errorf("Message is nil")
	}

	return types.NewCall(wr.conn.Metadata(), msg.Call, msg.Args...)
}

func (wr *ParachainWriter) makeHeaderImportCall(header *chain.Header) (types.Call, error) {
	if header == (*chain.Header)(nil) {
		return types.Call{}, fmt.Errorf("Header is nil")
	}

	return types.NewCall(wr.conn.Metadata(), "EthereumLightClient.import_header", header.HeaderData, header.ProofData)
}

func (wr *ParachainWriter) queryImportedHeaderExists(hash types.H256) (bool, error) {
	key, err := types.CreateStorageKey(wr.conn.Metadata(), "EthereumLightClient", "Headers", hash[:], nil)
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
