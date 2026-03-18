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

// RetryOnErrorSubstring retries fn when err.Error() contains retryableSubstring.
// It uses exponential backoff starting at DefaultRetryInitialBackoff and capped at DefaultRetryMaxBackoff.
// After DefaultRetryMaxRetries attempts, it returns the last error.
func RetryOnErrorSubstring(ctx context.Context, logger *log.Entry, retryableSubstring string, fn func() error) error {
	backoff := DefaultRetryInitialBackoff
	for attempt := 1; attempt <= DefaultRetryMaxRetries; attempt++ {
		err := fn()
		if err == nil {
			return nil
		}
		if retryableSubstring == "" || !strings.Contains(err.Error(), retryableSubstring) {
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
