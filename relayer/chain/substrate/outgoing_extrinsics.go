// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"
)

const MaxWatchedExtrinsics = 100

type extrinsicPool struct {
	sync.Mutex
	conn    *Connection
	eg      *errgroup.Group
	log     *logrus.Entry
	watched uint
}

func newExtrinsicPool(eg *errgroup.Group, conn *Connection, log *logrus.Entry) *extrinsicPool {
	ep := extrinsicPool{
		conn:    conn,
		eg:      eg,
		log:     log,
		watched: 0,
	}
	return &ep
}

func (ep *extrinsicPool) WaitForSubmitAndWatch(ctx context.Context, ext *types.Extrinsic) {
	defer ep.Unlock()

	for {
		ep.Lock()
		if ep.hasCapacity() {
			ep.eg.Go(func() error {
				return ep.submitAndWatchLoop(ctx, ext)
			})

			ep.watched++
			return
		}
		ep.Unlock()

		time.Sleep(100 * time.Millisecond)
	}
}

func (ep *extrinsicPool) hasCapacity() bool {
	return ep.watched < MaxWatchedExtrinsics
}

func (ep *extrinsicPool) submitAndWatchLoop(ctx context.Context, ext *types.Extrinsic) error {
	sub, err := ep.conn.api.RPC.Author.SubmitAndWatchExtrinsic(*ext)
	if err != nil {
		return err
	}

	for {
		select {
		case <-ctx.Done():
			return fmt.Errorf("Context was canceled. Stopping extrinsic monitoring")
		case status := <-sub.Chan():
			ep.logStatusUpdate(status)

			// Indicates that the extrinsic wasn't processed. We need to retry
			if status.IsDropped || status.IsInvalid {
				sub.Unsubscribe()
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
				ep.watched--
				return nil
			}
		case err := <-sub.Err():
			return err
		}
	}
}

func (ep *extrinsicPool) logStatusUpdate(status types.ExtrinsicStatus) {
	statusBytes, _ := status.MarshalJSON()
	ep.log.WithField("status", string(statusBytes)).Debug("Received extrinsic status update")
}
