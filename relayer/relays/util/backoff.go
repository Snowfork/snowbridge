package util

import (
	"context"
	"errors"
	"math/rand"
	"time"

	log "github.com/sirupsen/logrus"
)

// ScheduleConfig configures the randomized backoff algorithm for multi-operator coordination.
// Multiple relayers can run with identical configuration - coordination happens via
// random backoff timing and on-chain idempotency checks.
type ScheduleConfig struct {
	// Minimum backoff before first relay attempt (seconds)
	MinBackoffSeconds uint64 `mapstructure:"minBackoffSeconds"`
	// Maximum backoff before first relay attempt (seconds)
	// Actual backoff is random in [MinBackoffSeconds, MaxBackoffSeconds]
	MaxBackoffSeconds uint64 `mapstructure:"maxBackoffSeconds"`
	// Additional random jitter added to backoff (milliseconds)
	JitterMs uint64 `mapstructure:"jitterMs"`
	// Maximum number of messages to process concurrently
	MaxParallelMessages uint64 `mapstructure:"maxParallelMessages"`
}

// Validate checks that the schedule configuration is valid.
func (c ScheduleConfig) Validate() error {
	if c.MinBackoffSeconds == 0 && c.MaxBackoffSeconds == 0 {
		return errors.New("schedule config not set: minBackoffSeconds and maxBackoffSeconds are required")
	}
	if c.MaxBackoffSeconds < c.MinBackoffSeconds {
		return errors.New("maxBackoffSeconds must be >= minBackoffSeconds")
	}
	if c.MaxParallelMessages == 0 {
		return errors.New("maxParallelMessages must be > 0")
	}
	return nil
}

// RandomBackoff returns a random duration within the configured backoff range plus jitter.
// This is used for multi-operator coordination where each operator independently
// waits a random amount of time before attempting to relay a message.
func (c ScheduleConfig) RandomBackoff() time.Duration {
	var backoffSeconds uint64
	if c.MaxBackoffSeconds <= c.MinBackoffSeconds {
		backoffSeconds = c.MinBackoffSeconds
	} else {
		backoffSeconds = c.MinBackoffSeconds + uint64(rand.Int63n(int64(c.MaxBackoffSeconds-c.MinBackoffSeconds+1)))
	}

	var jitterMs int64 = 0
	if c.JitterMs > 0 {
		jitterMs = rand.Int63n(int64(c.JitterMs + 1))
	}

	return time.Duration(backoffSeconds)*time.Second + time.Duration(jitterMs)*time.Millisecond
}

// WaitWithBackoff waits for a random backoff duration, respecting context cancellation.
// Returns nil if the wait completed, or the context error if cancelled.
func (c ScheduleConfig) WaitWithBackoff(ctx context.Context) error {
	backoff := c.RandomBackoff()
	log.WithField("backoff", backoff).Debug("waiting with random backoff")

	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-time.After(backoff):
		return nil
	}
}
