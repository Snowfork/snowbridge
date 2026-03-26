package error_tracking

import (
	"time"

	log "github.com/sirupsen/logrus"
)

type RetryConfig struct {
	MaxRetries      int
	InitialDelay    time.Duration
	BackoffMultiple float64
}

func DefaultRetryConfig() RetryConfig {
	return RetryConfig{
		MaxRetries:      3,
		InitialDelay:    5 * time.Second,
		BackoffMultiple: 2.0,
	}
}

func RetryWithTracking(
	tracker *ErrorTracker,
	config RetryConfig,
	operation func() error,
	logFields log.Fields,
) error {
	retryDelay := config.InitialDelay

	for attempt := 0; attempt <= config.MaxRetries; attempt++ {
		err := operation()

		// Success
		if err == nil {
			tracker.RecordSuccess()
			return nil
		}

		// Non-retryable error
		if !IsTransientError(err) {
			log.WithFields(log.Fields{
				"error": err,
			}).Error("permanent error encountered")
			tracker.RecordPermanentError()
			return err
		}

		// Don't retry on last attempt
		if attempt == config.MaxRetries {
			tracker.RecordTransientError()
			log.WithFields(log.Fields{
				"error":   err,
				"attempt": attempt + 1,
			}).Error("final retry attempt failed")
			return err
		}

		// Log retry and wait
		log.WithFields(log.Fields{
			"error":   err,
			"attempt": attempt + 1,
			"retryIn": retryDelay,
		}).Warn("transient error detected, retrying")

		time.Sleep(retryDelay)
		// Exponential backoff
		retryDelay = time.Duration(float64(retryDelay) * config.BackoffMultiple)
	}

	return nil // Should never reach here
}
