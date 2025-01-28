package beefy

import (
	"context"
	"sync/atomic"
	"time"
)

type TokenBucket struct {
	tokens atomic.Uint64
	// Maximum number of tokens available to consume
	maxTokens uint64
	// The number of tokens added each refill period
	refillAmount uint64
	// The refill period
	refillPeriod time.Duration
}

func NewTokenBucket(maxTokens, refillAmount uint64, refillPeriod time.Duration) *TokenBucket {
	tb := &TokenBucket{
		maxTokens:    maxTokens,
		refillAmount: refillAmount,
		refillPeriod: refillPeriod,
	}
	tb.tokens.Store(maxTokens)
	return tb
}

func (tb *TokenBucket) Start(ctx context.Context) {
	go tb.refiller(ctx)
}

func (tb *TokenBucket) refiller(ctx context.Context) {
	ticker := time.NewTicker(tb.refillPeriod)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			currentTokens := tb.tokens.Load()
			newTokens := currentTokens + tb.refillAmount
			if newTokens > tb.maxTokens {
				newTokens = tb.maxTokens
			}
			tb.tokens.Store(newTokens)
		}
	}
}

func (tb *TokenBucket) TryConsume(tokens uint64) bool {
	for {
		currentTokens := tb.tokens.Load()
		if currentTokens < tokens {
			return false
		}

		if tb.tokens.CompareAndSwap(currentTokens, currentTokens-tokens) {
			return true
		}
	}
}
