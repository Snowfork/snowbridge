// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"
)

const MaxWatchedExtrinsics = 20

type ExtrinsicPool struct {
	sync.Mutex
	conn     *Connection
	eg       *errgroup.Group
	log      *logrus.Entry
	maxNonce uint32
	watched  chan struct{}
}

func NewExtrinsicPool(eg *errgroup.Group, conn *Connection, log *logrus.Entry) *ExtrinsicPool {
	ep := ExtrinsicPool{
		conn:    conn,
		eg:      eg,
		log:     log,
		watched: make(chan struct{}, MaxWatchedExtrinsics),
	}
	return &ep
}

func (ep *ExtrinsicPool) WaitForSubmitAndWatch(ctx context.Context, nonce uint32, ext *types.Extrinsic, onProcessed func() error) {
	select {
	case ep.watched <- struct{}{}:
		ep.eg.Go(func() error {
			return ep.submitAndWatchLoop(ctx, nonce, ext, onProcessed)
		})
	case <-ctx.Done():
	}
}

func (ep *ExtrinsicPool) submitAndWatchLoop(ctx context.Context, nonce uint32, ext *types.Extrinsic, onProcessed func() error) error {
	sub, err := ep.conn.api.RPC.Author.SubmitAndWatchExtrinsic(*ext)
	if err != nil {
		return err
	}

	for {
		select {
		case <-ctx.Done():
			return fmt.Errorf("Context was canceled. Stopping extrinsic monitoring")

		case status := <-sub.Chan():
			if status.IsDropped || status.IsInvalid {
				// Indicates that the extrinsic wasn't processed. We expect the Substrate txpool to be
				// stuck until this nonce is successfully provided. But it might be provided without this
				// relayer's intervention, e.g. if an internal Substrate mechanism re-introduces it to the
				// txpool.
				sub.Unsubscribe()
				statusStr := ep.getStatusString(&status)
				ep.log.WithFields(logrus.Fields{
					"nonce":  nonce,
					"status": statusStr,
				}).Debug("Extrinsic failed to be processed")

				// Back off for ~1 block to give the txpool time to resolve any backlog
				time.Sleep(time.Second * 6)

				ep.Lock()
				if nonce <= ep.maxNonce {
					// We're in the clear - no need to retry
					<-ep.watched
					ep.Unlock()
					return nil
				}
				ep.Unlock()

				// This might fail because the transaction has been temporarily banned in Substrate. In that
				// case it's best to crash, wait a while and restart the relayer.
				ep.log.WithFields(logrus.Fields{
					"nonce":  nonce,
					"status": statusStr,
				}).Debug("Re-submitting failed extrinsic")
				newSub, err := ep.conn.api.RPC.Author.SubmitAndWatchExtrinsic(*ext)
				if err != nil {
					return err
				}
				sub = newSub

			} else if !status.IsReady && !status.IsFuture && !status.IsBroadcast {
				// We assume all other status codes indicate that the extrinsic was processed
				// and account nonce was incremented.
				// See details at:
				// https://github.com/paritytech/substrate/blob/29aca981db5e8bf8b5538e6c7920ded917013ef3/primitives/transaction-pool/src/pool.rs#L56-L127
				sub.Unsubscribe()
				ep.Lock()
				defer ep.Unlock()
				if nonce > ep.maxNonce {
					ep.maxNonce = nonce
				}
				<-ep.watched
				return onProcessed()
			}

		case err := <-sub.Err():
			return err
		}
	}
}

func (ep *ExtrinsicPool) getStatusString(status *types.ExtrinsicStatus) string {
	statusBytes, err := status.MarshalJSON()
	if err != nil {
		return "failed to serialize status"
	}
	return string(statusBytes)
}
