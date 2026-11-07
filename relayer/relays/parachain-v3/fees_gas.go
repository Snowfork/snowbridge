package parachainV3

// Relayer-side gas for fee estimation (not on-chain enforcement). Tune when gateway/beefy costs shift.
// Limit floor uses MaxDispatchGas; fee pricing uses BaseUnlockGas/BaseMintGas — see contracts/src/Gateway.sol.
import (
	"fmt"
	"log"
	"os"
	"strconv"

	"github.com/snowfork/snowbridge/relayer/prices"
)

const (
	relayBlockTimeSecs = 6 // nominal relay block time for RELAY_SKIP_MAX_LAG_SECS → blocks
)

var (
	// Conservative BEEFY floor for aggregate3 gas limit; OOG on nested v2_submit beats losing a race.
	BaseBeefyFiatShamirGas = getEnvUint64("BASE_BEEFY_FIAT_SHAMIR_GAS", 2_000_000)
	// Expected BEEFY burn for static fast-path profitability only; do not lower the limit floor above.
	BaseBeefyFiatShamirFeeGas = getEnvUint64("BASE_BEEFY_FIAT_SHAMIR_FEE_GAS", 1_850_000)
	BaseBeefyTwoPhaseGas       = getEnvUint64("BASE_BEEFY_TWO_PHASE_GAS", 1_000_000)
	BaseMessageVerificationGas = getEnvUint64("BASE_MESSAGE_VERIFICATION_GAS", 100_000)
	BaseUnlockGas              = getEnvUint64("BASE_UNLOCK_GAS", 80_000)
	BaseMintGas                = getEnvUint64("BASE_MINT_GAS", 60_000)
	BaseV1StaticRewardWei      = getEnvUint64("BASE_V1_STATIC_REWARD_WEI", 250_000_000_000_000) // 0.00025 ETH
	BaseV1StaticGas            = getEnvUint64("BASE_V1_STATIC_GAS", 150_000)
	HighGasPriceThresholdWei   = getEnvUint64("HIGH_GAS_PRICE_THRESHOLD_WEI", 250_000_000) // 0.25 gwei
	// Handover skip window must exceed PROFITABILITY_MAX_DELAY_SECS (checked in validateProfitabilityConfig).
	RelaySkipMaxLagSecs   = getEnvUint64("RELAY_SKIP_MAX_LAG_SECS", 28800) // 8 hours
	RelaySkipMaxLagBlocks = RelaySkipMaxLagSecs / relayBlockTimeSecs

	// Optional fork eth_call before V2 fast-path broadcast; default uses static gas × live base fee.
	FastPathDryRun = getEnvBool("FAST_PATH_DRY_RUN", false)
	// Inflates profitability cost for a full EIP-1559 base-fee uptick between submit and inclusion.
	FastPathBaseFeeMarginBps = getEnvUint64("FAST_PATH_BASE_FEE_MARGIN_BPS", 1250)
	FastPathCatchUpIntervalSecs = getEnvUint64("FAST_PATH_CATCHUP_INTERVAL_SECS", 120)
)

// Called at relay startup (not init) so bad env returns an error instead of import-time panic.
func validateProfitabilityConfig() error {
	if maxDelaySecs := prices.MaxDelaySecs(); RelaySkipMaxLagSecs <= maxDelaySecs {
		return fmt.Errorf("RELAY_SKIP_MAX_LAG_SECS (%d) must be greater than PROFITABILITY_MAX_DELAY_SECS (%d)",
			RelaySkipMaxLagSecs, maxDelaySecs)
	}
	if BaseBeefyFiatShamirFeeGas > BaseBeefyFiatShamirGas {
		return fmt.Errorf("BASE_BEEFY_FIAT_SHAMIR_FEE_GAS (%d) must be ≤ BASE_BEEFY_FIAT_SHAMIR_GAS (%d)",
			BaseBeefyFiatShamirFeeGas, BaseBeefyFiatShamirGas)
	}
	return nil
}

func getEnvUint64(key string, defaultVal uint64) uint64 {
	valStr := os.Getenv(key)
	if valStr == "" {
		return defaultVal
	}
	val, err := strconv.ParseUint(valStr, 10, 64)
	if err != nil {
		log.Printf("Invalid value for %s: %v, using default %d", key, err, defaultVal)
		return defaultVal
	}
	return val
}

func getEnvBool(key string, defaultVal bool) bool {
	valStr := os.Getenv(key)
	if valStr == "" {
		return defaultVal
	}
	val, err := strconv.ParseBool(valStr)
	if err != nil {
		log.Printf("Invalid value for %s: %v, using default %v", key, err, defaultVal)
		return defaultVal
	}
	return val
}

// V2 fast-path fee gas: BaseBeefyFiatShamirFeeGas plus messages (not the higher limit floor or multicall headroom).
func staticAggregate3FeeGas(minSubmitProofGasSum uint64) uint64 {
	return BaseBeefyFiatShamirFeeGas + multicall3AggregateOverheadBuffer + minSubmitProofGasSum
}

// Multicall3 gas heuristics; gatewayDispatchOverheadBuffer must match DISPATCH_OVERHEAD_GAS_V2 in contracts/src/Gateway.sol.
var (
	gatewayDispatchOverheadBuffer     = getEnvUint64("GATEWAY_DISPATCH_OVERHEAD_BUFFER", 24_000)
	multicall3AggregateOverheadBuffer = getEnvUint64("MULTICALL3_AGGREGATE_OVERHEAD_BUFFER", 120_000)
	// Headroom so sequential BEEFY + v2_submit still passes Gateway gasleft()*63/64; fast path uses AllowFailure=false.
	multicallBeefyGasHeadroom = getEnvUint64("MULTICALL_BEEFY_GAS_HEADROOM", 1_500_000)
	multicallMaxGasLimit      = getEnvUint64("MULTICALL_MAX_GAS_LIMIT", 8_000_000)
)
