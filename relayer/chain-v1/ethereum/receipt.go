// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"

	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rpc"
	log "github.com/sirupsen/logrus"
)

// GetAllReceipts fetches all the transaction receipts in the given block.
func GetAllReceipts(ctx context.Context, conn *Connection, block *etypes.Block) (etypes.Receipts, error) {
	blockHash := block.Hash()
	log.WithFields(log.Fields{
		"numTransactions": len(block.Body().Transactions),
		"blockHash":       blockHash,
	}).Debug("Querying transaction receipts")
	return conn.client.BlockReceipts(ctx, rpc.BlockNumberOrHashWithHash(blockHash, false))
}
