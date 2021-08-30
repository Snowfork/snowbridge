// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"context"
	"math/big"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"golang.org/x/sync/errgroup"
	"golang.org/x/sync/semaphore"
)
const MaxWatchedExtrinsics = 10

type ExtrinsicPool struct {
	conn     *Connection
	eg       *errgroup.Group
	sem  *semaphore.Weighted
}

func NewExtrinsicPool(eg *errgroup.Group, conn *Connection) *ExtrinsicPool {
	ep := ExtrinsicPool{
		conn:    conn,
		eg:      eg,
		sem: semaphore.NewWeighted(MaxWatchedExtrinsics),
	}
	return &ep
}

func (ep *ExtrinsicPool) WaitForSubmitAndWatch(ctx context.Context, ext *types.Extrinsic) error {
	err := ep.sem.Acquire(ctx, 1)
	if err != nil {
		return err
	}

	sub, err := ep.conn.api.RPC.Author.SubmitAndWatchExtrinsic(*ext)
	if err != nil {
		ep.sem.Release(1)
		return err
	}

	ep.eg.Go(func() error {
		for {
			select {
			case <-ctx.Done():
				return nil
			case err := <-sub.Err():
				log.WithError(err).WithField("nonce", nonce(ext)).Error("Subscription failed for extrinsic status")
				return err
			case status := <-sub.Chan():
				// https://github.com/paritytech/substrate/blob/29aca981db5e8bf8b5538e6c7920ded917013ef3/primitives/transaction-pool/src/pool.rs#L56-L127
				if status.IsInBlock || status.IsDropped || status.IsInvalid || status.IsUsurped {
					log.WithFields(log.Fields{
						"nonce":  nonce(ext),
						"reason": reason(&status),
					}).Debug("Extrinsic left the transaction pool")
					sub.Unsubscribe()
					ep.sem.Release(1)
					return nil
				}
			}
		}
	})

	return nil
}

func nonce(ext *types.Extrinsic) uint64 {
	nonce := big.Int(ext.Signature.Nonce)
	return nonce.Uint64()
}

func reason(status *types.ExtrinsicStatus) string {
	switch  {
	case status.IsInBlock:
		return "InBlock"
	case status.IsDropped:
		return "Dropped"
	case status.IsInvalid:
		return "Invalid"
	case status.IsUsurped:
		return "Usurped"
	}
	return ""
}
