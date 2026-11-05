package prices

import (
	"os"
	"strconv"

	log "github.com/sirupsen/logrus"
)

const (
	maxDelaySecsEnvKey  = "PROFITABILITY_MAX_DELAY_SECS"
	defaultMaxDelaySecs = 14400 // 4 hours

	rewardRateEnvKey          = "PROFITABILITY_REWARD_RATE"
	defaultRewardRate float64 = 0.0002 // 0.02%
)

// RewardRate returns the configured PROFITABILITY_REWARD_RATE — the required relayer reward as a
// fraction of the transferred USD value. Shared by both relay directions.
func RewardRate() float64 {
	valStr := os.Getenv(rewardRateEnvKey)
	if valStr == "" {
		return defaultRewardRate
	}
	val, err := strconv.ParseFloat(valStr, 64)
	if err != nil || val < 0 {
		log.Warnf("Invalid value for %s: %q, using default %g", rewardRateEnvKey, valStr, defaultRewardRate)
		return defaultRewardRate
	}
	return val
}

// MaxDelayBlocks converts the globally configured PROFITABILITY_MAX_DELAY_SECS
// into a block count for a chain with the given block time. Both relay
// directions share the same delay budget and each converts it using its own
// block time. A value of 0 disables the max-delay valve.
func MaxDelayBlocks(blockTimeSecs uint64) uint64 {
	if blockTimeSecs == 0 {
		return 0
	}
	return MaxDelaySecs() / blockTimeSecs
}

// MaxDelaySecs returns the globally configured PROFITABILITY_MAX_DELAY_SECS budget.
func MaxDelaySecs() uint64 {
	valStr := os.Getenv(maxDelaySecsEnvKey)
	if valStr == "" {
		return defaultMaxDelaySecs
	}
	secs, err := strconv.ParseUint(valStr, 10, 64)
	if err != nil {
		log.Warnf("Invalid value for %s: %q, using default %d", maxDelaySecsEnvKey, valStr, defaultMaxDelaySecs)
		return defaultMaxDelaySecs
	}
	return secs
}
