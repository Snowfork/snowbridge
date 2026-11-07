// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"
	"time"

	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rpc"
	log "github.com/sirupsen/logrus"
)

// GetAllReceipts fetches all the transaction receipts in the given block.
// Platåberget's public RPC frequently 502s or returns null for eth_getBlockReceipts, so this
// retries briefly before falling back to per-transaction eth_getTransactionReceipt. By-hash is
// tried first: it stays pinned to this exact block under a reorg, where by-number would
// silently return receipts for whatever block now sits at that height.
func GetAllReceipts(ctx context.Context, conn *Connection, block *etypes.Block) (etypes.Receipts, error) {
	blockHash := block.Hash()
	blockNumber := block.NumberU64()
	txs := block.Body().Transactions
	log.WithFields(log.Fields{
		"numTransactions": len(txs),
		"blockHash":       blockHash,
		"blockNumber":     blockNumber,
	}).Debug("Querying transaction receipts")

	tryBlockReceipts := func(id rpc.BlockNumberOrHash) (etypes.Receipts, error) {
		receipts, err := conn.client.BlockReceipts(ctx, id)
		if err != nil {
			return nil, err
		}
		if receipts == nil {
			return nil, fmt.Errorf("eth_getBlockReceipts returned null")
		}
		return receipts, nil
	}

	var lastErr error
	backoff := 2 * time.Second
	for attempt := 1; attempt <= 3; attempt++ {
		receipts, err := tryBlockReceipts(rpc.BlockNumberOrHashWithHash(blockHash, false))
		if err == nil {
			return receipts, nil
		}
		lastErr = err
		log.WithError(err).WithFields(log.Fields{
			"attempt":   attempt,
			"blockHash": blockHash,
		}).Warn("BlockReceipts-by-hash failed; retrying")

		receipts, err = tryBlockReceipts(rpc.BlockNumberOrHashWithNumber(rpc.BlockNumber(blockNumber)))
		if err == nil {
			return receipts, nil
		}
		lastErr = err
		log.WithError(err).WithFields(log.Fields{
			"attempt":     attempt,
			"blockNumber": blockNumber,
		}).Warn("BlockReceipts-by-number failed; retrying")

		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-time.After(backoff):
		}
		if backoff < 20*time.Second {
			backoff *= 2
		}
	}

	log.WithError(lastErr).WithFields(log.Fields{
		"blockHash":   blockHash,
		"blockNumber": blockNumber,
	}).Warn("BlockReceipts exhausted; falling back to per-transaction receipts")

	receipts := make(etypes.Receipts, 0, len(txs))
	for i, tx := range txs {
		var receipt *etypes.Receipt
		var err error
		txBackoff := time.Second
		for attempt := 1; attempt <= 8; attempt++ {
			receipt, err = conn.client.TransactionReceipt(ctx, tx.Hash())
			if err == nil && receipt != nil {
				break
			}
			if err == nil {
				err = fmt.Errorf("eth_getTransactionReceipt returned null for %s", tx.Hash().Hex())
			}
			log.WithError(err).WithFields(log.Fields{
				"attempt": attempt,
				"txHash":  tx.Hash().Hex(),
				"index":   i,
			}).Warn("TransactionReceipt failed; retrying")
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(txBackoff):
			}
			if txBackoff < 20*time.Second {
				txBackoff *= 2
			}
		}
		if err != nil || receipt == nil {
			return nil, fmt.Errorf("get receipt for tx %s (index %d): %w", tx.Hash().Hex(), i, err)
		}
		receipts = append(receipts, receipt)
	}
	return receipts, nil
}
