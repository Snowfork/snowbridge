// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	log "github.com/sirupsen/logrus"
)

const (
	// DefaultGetLogsBlockRange is the inclusive eth_getLogs window. Alchemy Free
	// rejects ranges larger than 10; paid RPCs can raise ETH_GETLOGS_BLOCK_RANGE.
	DefaultGetLogsBlockRange uint64 = 10
	GetLogsBlockRangeEnv            = "ETH_GETLOGS_BLOCK_RANGE"
	// GetLogsPageDelayEnv spaces consecutive eth_getLogs windows so a long backward scan
	// stays under per-second RPC limits (e.g. Alchemy compute units per second).
	GetLogsPageDelayEnv = "ETH_GETLOGS_PAGE_DELAY_MS"
)

var getLogsRangeLogOnce sync.Once

// ParseGetLogsBlockRange returns an inclusive block count. Empty or invalid
// values fall back to DefaultGetLogsBlockRange (Alchemy Free's max).
func ParseGetLogsBlockRange(raw string) uint64 {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return DefaultGetLogsBlockRange
	}
	n, err := strconv.ParseUint(raw, 10, 64)
	if err != nil || n == 0 {
		return DefaultGetLogsBlockRange
	}
	return n
}

// GetLogsBlockRange is the inclusive eth_getLogs window used by findEvents.
func GetLogsBlockRange() uint64 {
	n := ParseGetLogsBlockRange(os.Getenv(GetLogsBlockRangeEnv))
	getLogsRangeLogOnce.Do(func() {
		log.WithFields(log.Fields{
			"env":    GetLogsBlockRangeEnv,
			"blocks": n,
		}).Info("Using inclusive eth_getLogs block range")
	})
	return n
}

// ParseGetLogsPageDelay returns the pause between eth_getLogs windows. Empty or
// invalid values mean no pause.
func ParseGetLogsPageDelay(raw string) time.Duration {
	ms, err := strconv.ParseUint(strings.TrimSpace(raw), 10, 32)
	if err != nil {
		return 0
	}
	return time.Duration(ms) * time.Millisecond
}

// LogFilterStart is the inclusive start block for a window ending at endBlock
// with at most rangeSize blocks (inclusive). Alchemy's 10-block cap is
// [end-9, end], not [end-10, end] (11 blocks).
func LogFilterStart(endBlock, rangeSize uint64) uint64 {
	if rangeSize == 0 {
		rangeSize = DefaultGetLogsBlockRange
	}
	if endBlock+1 <= rangeSize {
		return 0
	}
	return endBlock - (rangeSize - 1)
}

// WalkLogFilterPages calls fn for each inclusive window walking backward from
// latestBlock until fn returns done or genesis is reached.
func WalkLogFilterPages(ctx context.Context, latestBlock uint64, fn func(opts *bind.FilterOpts) (done bool, err error)) error {
	rangeSize := GetLogsBlockRange()
	delay := ParseGetLogsPageDelay(os.Getenv(GetLogsPageDelayEnv))
	blockNumber := latestBlock
	for first := true; ; first = false {
		if !first && delay > 0 {
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-time.After(delay):
			}
		}
		begin := LogFilterStart(blockNumber, rangeSize)
		end := blockNumber
		opts := bind.FilterOpts{
			Start:   begin,
			End:     &end,
			Context: ctx,
		}
		done, err := fn(&opts)
		if err != nil {
			return err
		}
		if done || begin == 0 {
			return nil
		}
		blockNumber = begin - 1
	}
}

// NonceBlock returns the block in [from, to] that emitted nonce: the first block
// whose post-state outbound nonce is at least nonce. The Gateway increments the
// nonce and emits the event in the same call. from is a lower bound (e.g. the
// block of an earlier nonce); the search gallops back from to and then bisects,
// so its cost grows with the log of the distance from to.
func NonceBlock(
	ctx context.Context,
	from, to, nonce uint64,
	nonceAt func(ctx context.Context, block uint64) (uint64, error),
) (uint64, error) {
	n, err := nonceAt(ctx, to)
	if err != nil {
		return 0, err
	}
	if n < nonce {
		return 0, fmt.Errorf("nonce %d not emitted by block %d (outbound nonce %d)", nonce, to, n)
	}

	// Invariant: nonceAt(hi) >= nonce, and nonceAt(lo) < nonce once found.
	hi := to
	var lo uint64
	for step := uint64(1); ; step *= 2 {
		if hi <= from {
			return hi, nil
		}
		lo = from
		if hi-from > step {
			lo = hi - step
		}
		n, err = nonceAt(ctx, lo)
		if err != nil {
			return 0, err
		}
		if n < nonce {
			break
		}
		hi = lo
	}
	for hi-lo > 1 {
		mid := lo + (hi-lo)/2
		n, err = nonceAt(ctx, mid)
		if err != nil {
			return 0, err
		}
		if n >= nonce {
			hi = mid
		} else {
			lo = mid
		}
	}
	return hi, nil
}
