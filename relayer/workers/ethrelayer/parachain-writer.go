package ethrelayer

import (
	"context"
	"fmt"

	"github.com/sirupsen/logrus"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
)

type ParachainWriter struct {
	conn        *parachain.Connection
	messages    <-chan []chain.Message
	headers     <-chan chain.Header
	log         *logrus.Entry
	nonce       uint32
	pool        *parachain.ExtrinsicPool
	genesisHash types.Hash
}

func NewParachainWriter(
	conn *parachain.Connection,
	messages <-chan []chain.Message,
	headers <-chan chain.Header,
	log *logrus.Entry,
) *ParachainWriter {
	return &ParachainWriter{
		conn:     conn,
		messages: messages,
		headers:  headers,
		log:      log,
	}
}

func (wr *ParachainWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	cancelWithError := func(err error) error {
		// Ensures the context is canceled so that the channels below are
		// closed by the listener
		eg.Go(func() error { return err })

		wr.log.Info("Shutting down writer...")
		// Avoid deadlock if the listener is still trying to send to a channel
		if wr.messages != nil {
			for range wr.messages {
				wr.log.Debug("Discarded message")
			}
		}
		for range wr.headers {
			wr.log.Debug("Discarded header")
		}

		return err
	}

	nonce, err := wr.queryAccountNonce()
	if err != nil {
		return cancelWithError(err)
	}
	wr.nonce = nonce

	genesisHash, err := wr.conn.GetAPI().RPC.Chain.GetBlockHash(0)
	if err != nil {
		return cancelWithError(err)
	}
	wr.genesisHash = genesisHash

	wr.pool = parachain.NewExtrinsicPool(eg, wr.conn, wr.log)

	eg.Go(func() error {
		err := wr.writeLoop(ctx)
		if err != nil {
			return cancelWithError(err)
		}
		return nil
	})

	return nil
}

func (wr *ParachainWriter) queryAccountNonce() (uint32, error) {
	key, err := types.CreateStorageKey(wr.conn.GetMetadata(), "System", "Account", wr.conn.GetKeypair().PublicKey, nil)
	if err != nil {
		return 0, err
	}

	var accountInfo types.AccountInfo
	ok, err := wr.conn.GetAPI().RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil {
		return 0, err
	}
	if !ok {
		return 0, fmt.Errorf("no account info found for %s", wr.conn.GetKeypair().URI)
	}

	return uint32(accountInfo.Nonce), nil
}

func (wr *ParachainWriter) writeLoop(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case msgs, ok := <-wr.messages:
			if !ok {
				return nil
			}

			var concreteMsgs []*chain.EthereumOutboundMessage
			for _, msg := range msgs {
				cmsg, ok := msg.(*chain.EthereumOutboundMessage)
				if !ok {
					return fmt.Errorf("Invalid message")
				}
				concreteMsgs = append(concreteMsgs, cmsg)
			}

			err := wr.WriteMessages(ctx, concreteMsgs)
			if err != nil {
				wr.log.WithFields(logrus.Fields{
					"error": err,
				}).Error("Failure submitting message to substrate")
				return err
			}
		case header, ok := <-wr.headers:
			if !ok {
				return nil
			}

			err := wr.WriteHeader(ctx, &header)
			if err != nil {
				wr.log.WithFields(logrus.Fields{
					"blockNumber": header.HeaderData.(ethereum.Header).Number,
					"error":       err,
				}).Error("Failure submitting header to substrate")
				return err
			}
		}
	}
}

// Write submits a transaction to the chain
func (wr *ParachainWriter) write(ctx context.Context, c types.Call) error {
	ext := types.NewExtrinsic(c)

	latestHash, err := wr.conn.GetAPI().RPC.Chain.GetFinalizedHead()
	if err != nil {
		return err
	}

	latestBlock, err := wr.conn.GetAPI().RPC.Chain.GetBlock(latestHash)
	if err != nil {
		return err
	}

	era := parachain.NewMortalEra(uint64(latestBlock.Block.Header.Number))

	rv, err := wr.conn.GetAPI().RPC.State.GetRuntimeVersionLatest()
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
	err = extI.Sign(*wr.conn.GetKeypair(), o)
	if err != nil {
		return err
	}

	wr.pool.WaitForSubmitAndWatch(ctx, wr.nonce, &extI)

	wr.nonce = wr.nonce + 1

	return nil
}

func (wr *ParachainWriter) WriteMessages(ctx context.Context, msgs []*chain.EthereumOutboundMessage) error {
	for _, msg := range msgs {

		c, err := types.NewCall(wr.conn.GetMetadata(), msg.Call, msg.Args...)
		if err != nil {
			return err
		}

		err = wr.write(ctx, c)
		if err != nil {
			return err
		}
	}

	wr.log.WithField("count", len(msgs)).Info("Submitted messages to Substrate")

	return nil
}

// WriteHeader submits a "VerifierLightclient.import_header" call
func (wr *ParachainWriter) WriteHeader(ctx context.Context, header *chain.Header) error {
	c, err := types.NewCall(wr.conn.GetMetadata(), "VerifierLightclient.import_header", header.HeaderData, header.ProofData)
	if err != nil {
		return err
	}

	err = wr.write(ctx, c)
	if err != nil {
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"blockNumber": header.HeaderData.(ethereum.Header).Number,
	}).Info("Submitted header to Substrate")

	return nil
}
