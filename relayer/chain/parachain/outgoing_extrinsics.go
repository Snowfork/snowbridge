// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"context"
	"errors"
	"sync"
	"time"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"golang.org/x/sync/errgroup"
)
const MaxWatchedExtrinsics = 10

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

func (ep *ExtrinsicPool) WaitForSubmitAndWatch(ctx context.Context, nonce uint32, ext *types.Extrinsic) {
	select {
	case ep.watched <- struct{}{}:
		ep.eg.Go(func() error {
			err := ep.submitAndWatchLoop(ctx, nonce, ext)
			if err != nil {
				if errors.Is(err, context.Canceled) {
					return nil
				}
				return err
			}
			return nil
		})
	case <-ctx.Done():
	}
}

func (ep *ExtrinsicPool) submitAndWatchLoop(ctx context.Context, nonce uint32, ext *types.Extrinsic) error {
	sub, err := ep.conn.api.RPC.Author.SubmitAndWatchExtrinsic(*ext)
	if err != nil {
		ep.log.WithError(err).WithField("nonce", nonce).Debug("Failed to submit extrinsic")
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			ep.log.WithField("reason", ctx.Err()).WithField("nonce", nonce).Debug("Stopping monitoring extrinsic status")
			return ctx.Err()
		case err := <-sub.Err():
			ep.log.WithError(err).WithField("nonce", nonce).Error("Subscription failed for extrinsic status")
			return err
		case status := <-sub.Chan():
			// https://github.com/paritytech/substrate/blob/29aca981db5e8bf8b5538e6c7920ded917013ef3/primitives/transaction-pool/src/pool.rs#L56-L127
			if status.IsDropped || status.IsInvalid || status.IsUsurped {
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
				time.Sleep(time.Second * 12)

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
					ep.log.WithError(err).WithField("nonce", nonce).Debug("Failed to submit extrinsic")
					return err
				}
				sub = newSub
			} else if status.IsInBlock {
				ep.log.WithFields(logrus.Fields{
					"nonce":  nonce,
					"blockHash": status.AsInBlock.Hex(),
				}).Debug("Extrinsic included in block")
				sub.Unsubscribe()
				ep.Lock()
				defer ep.Unlock()
				if nonce > ep.maxNonce {
					ep.maxNonce = nonce
				}
				<-ep.watched
				return nil
			}
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
