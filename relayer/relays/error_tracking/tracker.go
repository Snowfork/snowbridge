package error_tracking

import (
	"strings"
	"time"

	log "github.com/sirupsen/logrus"
)

type ErrorTracker struct {
	transientErrors   int
	permanentErrors   int
	totalAttempts     int
	lastResetTime     time.Time
	consecutiveErrors int
	maxConsecutive    int
}

func NewErrorTracker(maxConsecutive int) *ErrorTracker {
	return &ErrorTracker{
		lastResetTime:  time.Now(),
		maxConsecutive: maxConsecutive,
	}
}

func (et *ErrorTracker) RecordSuccess() {
	et.totalAttempts++
	et.consecutiveErrors = 0
	et.checkAndResetCounters()
}

func (et *ErrorTracker) RecordTransientError() {
	et.totalAttempts++
	et.transientErrors++
	et.consecutiveErrors++
	et.checkAndResetCounters()
	et.checkForAlerts()
}

func (et *ErrorTracker) RecordPermanentError() {
	et.totalAttempts++
	et.permanentErrors++
	et.consecutiveErrors = 0 // Permanent errors don't count for consecutive tracking
	et.checkAndResetCounters()
}

func (et *ErrorTracker) GetStats() (transient, permanent, total int, transientRate, permanentRate float64) {
	if et.totalAttempts == 0 {
		return et.transientErrors, et.permanentErrors, et.totalAttempts, 0.0, 0.0
	}

	transientRate = float64(et.transientErrors) / float64(et.totalAttempts)
	permanentRate = float64(et.permanentErrors) / float64(et.totalAttempts)

	return et.transientErrors, et.permanentErrors, et.totalAttempts, transientRate, permanentRate
}

func (et *ErrorTracker) checkAndResetCounters() {
	if time.Since(et.lastResetTime) > time.Hour {
		et.logHourlyStats()
		et.transientErrors = 0
		et.permanentErrors = 0
		et.totalAttempts = 0
		et.lastResetTime = time.Now()
	}
}

func (et *ErrorTracker) checkForAlerts() {
	// Alert on consecutive transient errors
	if et.consecutiveErrors >= et.maxConsecutive {
		log.WithFields(log.Fields{
			"consecutiveErrors": et.consecutiveErrors,
			"threshold":         et.maxConsecutive,
		}).Error("ALERT: High consecutive transient error rate detected - possible systemic issue")
	}

	// Alert on high error rate (>50% failure in current hour)
	if et.totalAttempts >= 20 { // Only alert after sufficient sample size
		errorRate := float64(et.transientErrors+et.permanentErrors) / float64(et.totalAttempts)
		if errorRate > 0.5 {
			log.WithFields(log.Fields{
				"errorRate":       errorRate,
				"transientErrors": et.transientErrors,
				"permanentErrors": et.permanentErrors,
				"totalAttempts":   et.totalAttempts,
			}).Error("ALERT: High error rate detected - over 50% of messages failing")
		}
	}
}

func (et *ErrorTracker) logHourlyStats() {
	if et.totalAttempts > 0 {
		transientRate := float64(et.transientErrors) / float64(et.totalAttempts)
		permanentRate := float64(et.permanentErrors) / float64(et.totalAttempts)

		log.WithFields(log.Fields{
			"totalAttempts":   et.totalAttempts,
			"transientErrors": et.transientErrors,
			"permanentErrors": et.permanentErrors,
			"transientRate":   transientRate,
			"permanentRate":   permanentRate,
			"consecutiveMax":  et.consecutiveErrors,
		}).Info("Hourly error statistics")
	}
}

func IsTransientError(err error) bool {
	if err == nil {
		return false
	}

	errStr := strings.ToLower(err.Error())

	transientPatterns := []string{
		"connection", "timeout", "deadline exceeded", "context deadline exceeded",
		"network", "refused", "unreachable", "unavailable", "busy",
		"temporary", "try again", "rate limit", "too many requests",
		"rpc", "dial", "i/o timeout", "broken pipe", "reset by peer",
		"finality timeout", "dropped", "invalid", "usurped",
	}

	for _, pattern := range transientPatterns {
		if strings.Contains(errStr, pattern) {
			return true
		}
	}

	return false
}
