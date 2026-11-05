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

var SnowbridgeTransientError = "snowbridge_transient_error"

// permanentPatterns are checked before transient patterns. Some permanent
// errors include substrings that would otherwise match a transient pattern —
// notably a wasm panic stderr containing "wasm `unreachable` instruction"
// would be matched by the network "unreachable" transient pattern. Patterns
// listed here win and force a permanent classification.
var permanentPatterns = lowered([]string{
	"wasm trap",
	"wasm `unreachable`",
	"rust_begin_unwind",
	"panic_fmt",
	"fungible asset of zero amount is not allowed",
	// ethereum-v3: Asset Hub dry_run_xcm rejected the destination XCM.
	"asset hub dry-run rejected",
})

// staleCodegenPattern identifies a gas estimator compiled against outdated
// Asset Hub / Bridge Hub subxt metadata after a runtime upgrade. Classified
// transient so nonces are not dead-lettered, but it never self-heals: the fix
// is rebuilding the relayer image with fresh metadata (release-relayer).
const staleCodegenPattern = "incompatiblecodegen"

// IsStaleMetadataError reports whether err is the stale-subxt-codegen failure,
// so callers can raise a dedicated alert: unlike other transient errors it
// requires operator action (image rebuild + redeploy) to resolve.
func IsStaleMetadataError(err error) bool {
	if err == nil {
		return false
	}
	return strings.Contains(strings.ToLower(err.Error()), staleCodegenPattern)
}

var transientPatterns = lowered([]string{
	SnowbridgeTransientError, "connection timeout", "deadline exceeded", "context deadline exceeded",
	"refused", "unreachable", "unavailable", "try again", "rate limit", "too many requests",
	"status code 429", "http 429",
	"i/o timeout", "broken pipe", "reset by peer", "finality timeout",
	"close sent", "closed network connection", "websocket: close",
	// go-ethereum rpc.Client after Close() (heartbeat Reconnect); distinct from
	// "websocket: close …" which is a still-open socket receiving a close frame.
	"client is closed",
	"fetch relaychain block", "fetch parachain block", "fetch relaychain inclusion block",
	"fetch block hash", "fetch block header", "fetch beefy finalized head",
	"fetch head for parachain", "fetch parachain block hash",
	"name or service not known", "no such host", "failed to lookup",
	"failed to resolve", "cannot connect", "connection error", "server misbehaving",
	"invalid transaction",
	"wait receipt timeout",
	// Platåberget / sparse Gloas testnets: FFG-proving attested slot or sync
	// supermajority may not exist yet; retry next ticker instead of exiting.
	"no attested header proving checkpoint",
	"no valid slot pair within",
	"matching block included not found",
	"sync committee at slot not supermajority",
	// Eth log filter queries are often rejected by flaky/limit-capped RPCs and
	// recover next interval (or after endpoint rotation on reconnect).
	"block range is too large",
	"block range extends beyond",
	"query returned more than",
	"response size exceeded",
	// Alchemy Free: eth_getLogs max 10-block range (wording is not "too large").
	"10 block range",
	"this block range should work",
	"under the free tier",
	// drpc free plan: HTTP 408 on slow eth_getLogs; retry next interval instead of exiting.
	"408 request timeout",
	// Gas estimator compiled against outdated subxt metadata after AH/BH upgrade
	// (see IsStaleMetadataError); rebuild image via release-relayer to actually heal.
	staleCodegenPattern,
})

func lowered(patterns []string) []string {
	out := make([]string, len(patterns))
	for i, p := range patterns {
		out[i] = strings.ToLower(p)
	}
	return out
}

func IsTransientError(err error) bool {
	if err == nil {
		return false
	}

	errStr := strings.ToLower(err.Error())

	for _, pattern := range permanentPatterns {
		if strings.Contains(errStr, pattern) {
			return false
		}
	}

	for _, pattern := range transientPatterns {
		if strings.Contains(errStr, pattern) {
			return true
		}
	}

	return false
}
