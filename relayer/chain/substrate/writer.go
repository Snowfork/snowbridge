// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"context"
	"encoding/hex"
	"fmt"
	"time"

	"github.com/sirupsen/logrus"

	"golang.org/x/sync/errgroup"

	"github.com/centrifuge/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
)

type Writer struct {
	conn     *Connection
	messages <-chan chain.Message
	headers  <-chan chain.Header
	log      *logrus.Entry
	nonce    uint32
}

func NewWriter(conn *Connection, messages <-chan chain.Message, headers <-chan chain.Header, log *logrus.Entry) (*Writer, error) {
	return &Writer{
		conn:     conn,
		messages: messages,
		headers:  headers,
		log:      log,
		nonce:    0,
	}, nil
}

func (wr *Writer) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return wr.writeLoop(ctx)
	})
	return nil
}

func (wr *Writer) onDone(ctx context.Context) error {
	wr.log.Info("Shutting down writer...")
	// Avoid deadlock if a listener is still trying to send to a channel
	if wr.messages != nil {
		for range wr.messages {
			wr.log.Debug("Discarded message")
		}
	}
	for range wr.headers {
		wr.log.Debug("Discarded header")
	}
	return ctx.Err()
}

func (wr *Writer) writeLoop(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return wr.onDone(ctx)
		case msg := <-wr.messages:
			err := wr.WriteMessage(ctx, &msg)
			if err != nil {
				wr.log.WithFields(logrus.Fields{
					"appid": hex.EncodeToString(msg.AppID[:]),
					"error": err,
				}).Error("Failure submitting message to substrate")
			}
		case header := <-wr.headers:
			err := wr.WriteHeader(ctx, &header)
			if err != nil {
				wr.log.WithFields(logrus.Fields{
					"blockNumber": header.HeaderData.(ethereum.Header).Number,
					"error":       err,
				}).Error("Failure submitting header to substrate")
			}
		}
	}
}

// Write submits a transaction to the chain
func (wr *Writer) write(ctx context.Context, c types.Call) error {

	ext := types.NewExtrinsic(c)

	era := types.ExtrinsicEra{IsMortalEra: false}

	genesisHash, err := wr.conn.api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}

	rv, err := wr.conn.api.RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		return err
	}

	key, err := types.CreateStorageKey(&wr.conn.metadata, "System", "Account", wr.conn.kp.PublicKey, nil)
	if err != nil {
		return err
	}

	// Poll account state until we have a new nonce. This matches our extrinsic
	// sending rate with the node's extrinsic processing rate, and avoids
	// the transaction being dropped from the node's transaction pool.
	var nonce uint32
	for {
		var accountInfo types.AccountInfo
		ok, err := wr.conn.api.RPC.State.GetStorageLatest(key, &accountInfo)
		if err != nil {
			return err
		}
		if !ok {
			return fmt.Errorf("no account info found for %s", wr.conn.kp.URI)
		}

		nonce = uint32(accountInfo.Nonce)
		if nonce != wr.nonce {
			wr.nonce = nonce
			break
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(time.Duration(1) * time.Second):
		}
	}

	o := types.SignatureOptions{
		BlockHash:          genesisHash,
		Era:                era,
		GenesisHash:        genesisHash,
		Nonce:              types.NewUCompactFromUInt(uint64(nonce)),
		SpecVersion:        rv.SpecVersion,
		Tip:                types.NewUCompactFromUInt(0),
		TransactionVersion: 1,
	}

	extI := ext

	err = extI.Sign(*wr.conn.kp, o)
	if err != nil {
		return err
	}

	_, err = wr.conn.api.RPC.Author.SubmitExtrinsic(extI)
	if err != nil {
		return err
	}

	return nil
}

// WriteMessage submits a "Bridge.submit" call
func (wr *Writer) WriteMessage(ctx context.Context, msg *chain.Message) error {
	c, err := types.NewCall(&wr.conn.metadata, "Bridge.submit", msg.AppID, msg.Payload)
	if err != nil {
		return err
	}

	err = wr.write(ctx, c)
	if err != nil {
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"appid": hex.EncodeToString(msg.AppID[:]),
	}).Info("Submitted message to Substrate")

	return nil
}

// WriteHeader submits a "VerifierLightclient.import_header" call
func (wr *Writer) WriteHeader(ctx context.Context, header *chain.Header) error {
	c, err := types.NewCall(&wr.conn.metadata, "VerifierLightclient.import_header", header.HeaderData, header.ProofData)
	if err != nil {
		return err
	}

	retries := 0
	for {
		err := wr.write(ctx, c)
		if err != nil {
			wr.log.WithFields(logrus.Fields{
				"blockNumber": header.HeaderData.(ethereum.Header).Number,
				"error":       err,
				"retries":     retries,
			}).Error("Failure submitting header to substrate")
		} else {
			break
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(getHeaderBackoffDelay(retries)):
		}
		retries = retries + 1
	}

	wr.log.WithFields(logrus.Fields{
		"blockNumber": header.HeaderData.(ethereum.Header).Number,
	}).Info("Submitted header to Substrate")

	return nil
}

func getHeaderBackoffDelay(retryCount int) time.Duration {
	backoff := []int{1, 1, 5, 5, 30, 30, 60} // seconds

	if retryCount < len(backoff)-1 {
		return time.Duration(backoff[retryCount]) * time.Second
	}
	return time.Duration(backoff[len(backoff)-1]) * time.Second
}
