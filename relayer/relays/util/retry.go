package util

import (
	"context"
	"strings"
	"time"

	log "github.com/sirupsen/logrus"
)

const (
	DefaultRetryInitialBackoff = 6 * time.Second
	DefaultRetryMaxBackoff     = 2 * time.Minute
	DefaultRetryMaxRetries     = 6
)

const DefaultRetryableSubstring = "504 Gateway Timeout"

// DefaultRetryableSubstrings is a broader set of transient RPC error patterns.
// It mirrors the matching done in ethereum.LikelyTransientRPCError (timeouts / gateway errors).
var DefaultRetryableSubstrings = []string{
	"504",
	"gateway timeout",
	"deadline exceeded",
	"i/o timeout",
	"connection reset",
}

// RetryOnErrorSubstring retries fn when err.Error() contains retryableSubstring.
// It uses exponential backoff starting at DefaultRetryInitialBackoff and capped at DefaultRetryMaxBackoff.
// After DefaultRetryMaxRetries attempts, it returns the last error.
func RetryOnErrorSubstring(ctx context.Context, logger *log.Entry, retryableSubstring string, fn func() error) error {
	return RetryOnErrorSubstrings(ctx, logger, []string{retryableSubstring}, fn)
}

// RetryOnErrorSubstrings retries fn when err.Error() contains any substring in retryableSubstrings.
// It uses exponential backoff starting at DefaultRetryInitialBackoff and capped at DefaultRetryMaxBackoff.
// After DefaultRetryMaxRetries attempts, it returns the last error.
func RetryOnErrorSubstrings(ctx context.Context, logger *log.Entry, retryableSubstrings []string, fn func() error) error {
	backoff := DefaultRetryInitialBackoff
	for attempt := 1; attempt <= DefaultRetryMaxRetries; attempt++ {
		err := fn()
		if err == nil {
			return nil
		}
		if len(retryableSubstrings) == 0 {
			return err
		}

		low := strings.ToLower(err.Error())
		retryable := false
		for _, sub := range retryableSubstrings {
			if sub == "" {
				continue
			}
			if strings.Contains(low, strings.ToLower(sub)) {
				retryable = true
				break
			}
		}
		if !retryable {
			return err
		}
		if attempt == DefaultRetryMaxRetries {
			return err
		}

		if logger != nil {
			logger.WithError(err).WithFields(log.Fields{
				"attempt": attempt,
				"backoff": backoff.String(),
			}).Warn("Transient error while writing message; retrying")
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(backoff):
		}

		if backoff < DefaultRetryMaxBackoff {
			backoff *= 2
			if backoff > DefaultRetryMaxBackoff {
				backoff = DefaultRetryMaxBackoff
			}
		}
	}
	return nil
}
