package parachainV3

// Relayer-side gas units for fee / profitability estimation (not enforced on-chain).
// When gateway or beefy costs change meaningfully, update these and release.
//
// BaseMessageVerificationGas is the v2_submit verification leg (MMR/header checks before dispatch).
// minGasForV2SubmitProof uses MaxDispatchGas for the tx gas-limit floor; estimatedGasForV2SubmitProof /
// MessageMinFeeWei use BaseUnlockGas / BaseMintGas for expected burn. Those pair with command kinds in
// contracts/src/Gateway.sol.
import (
	"fmt"
	"log"
	"os"
	"strconv"

	"github.com/snowfork/snowbridge/relayer/prices"
)

const (
	// relayBlockTimeSecs is the nominal relay-chain block time, used to convert
	// RELAY_SKIP_MAX_LAG_SECS into a relay block count.
	relayBlockTimeSecs = 6
)

var (
	// BaseBeefyFiatShamirGas is the BEEFY component of the aggregate3 tx gas-limit floor
	// (see multicallGasFloor). Keep this conservative; OOG on nested v2_submit is worse than
	// skipping a race.
	BaseBeefyFiatShamirGas = getEnvUint64("BASE_BEEFY_FIAT_SHAMIR_GAS", 2_000_000)
	// BaseBeefyFiatShamirFeeGas is the BEEFY component for V2 fast-path static profitability
	// only (see staticAggregate3FeeGas). Calibrated from recent fiat-shamir Multicall3
	// receipts (gasUsed p50 ≈ 2.07M for BEEFY+unlock ⇒ implied BEEFY ≈ 1.72–1.75M); 1.85M
	// leaves ~5% cushion vs p50 without the old ~12% overestimate that skipped nonce 808.
	// Limit floor stays on BaseBeefyFiatShamirGas — do not lower that for fee tuning.
	BaseBeefyFiatShamirFeeGas = getEnvUint64("BASE_BEEFY_FIAT_SHAMIR_FEE_GAS", 1_850_000)
	BaseBeefyTwoPhaseGas       = getEnvUint64("BASE_BEEFY_TWO_PHASE_GAS", 1_000_000)
	BaseMessageVerificationGas = getEnvUint64("BASE_MESSAGE_VERIFICATION_GAS", 100_000)
	BaseUnlockGas              = getEnvUint64("BASE_UNLOCK_GAS", 80_000)
	BaseMintGas                = getEnvUint64("BASE_MINT_GAS", 60_000)
	BaseV1StaticRewardWei      = getEnvUint64("BASE_V1_STATIC_REWARD_WEI", 250_000_000_000_000) // 0.00025 ETH
	BaseV1StaticGas            = getEnvUint64("BASE_V1_STATIC_GAS", 150_000)
	HighGasPriceThresholdWei   = getEnvUint64("HIGH_GAS_PRICE_THRESHOLD_WEI", 250_000_000) // 0.25 gwei
	// RelaySkipMaxLagSecs is how far the light client may lag before a non-urgent handover is no
	// longer skipped on high gas. Must exceed PROFITABILITY_MAX_DELAY_SECS (enforced in init).
	RelaySkipMaxLagSecs   = getEnvUint64("RELAY_SKIP_MAX_LAG_SECS", 28800) // 8 hours
	RelaySkipMaxLagBlocks = RelaySkipMaxLagSecs / relayBlockTimeSecs

	// FastPathDryRun enables the forked-mainnet aggregate3 eth_call before broadcast on the
	// V2-only fast path. Default off: hot path uses static gas × live base fee instead.
	FastPathDryRun = getEnvBool("FAST_PATH_DRY_RUN", false)
	// FastPathBaseFeeMarginBps inflates the fast-path profitability cost basis so a full
	// EIP-1559 max base-fee uptick (12.5% per block) between submit and next-block inclusion
	// still break-even. Default 1250 = 12.5%. Direct-relay uses its own dry-run gate.
	FastPathBaseFeeMarginBps = getEnvUint64("FAST_PATH_BASE_FEE_MARGIN_BPS", 1250)
	// FastPathCatchUpIntervalSecs is the catch-up ticker interval for the single fastpath
	// service (direct-relay / missed-WS / async handover). Default 120s — catch-up is a
	// safety net behind WS instant, not the hot path.
	FastPathCatchUpIntervalSecs = getEnvUint64("FAST_PATH_CATCHUP_INTERVAL_SECS", 120)
)

// validateProfitabilityConfig checks the profitability/gas env invariants. It is called at
// startup (not in init) so misconfiguration surfaces as a returned error rather than crashing
// the process on package import.
func validateProfitabilityConfig() error {
	// The handover skip window must outlast the relay deadline; otherwise a message could be both
	// past its max-delay valve and still skipped on gas, stranding it.
	if maxDelaySecs := prices.MaxDelaySecs(); RelaySkipMaxLagSecs <= maxDelaySecs {
		return fmt.Errorf("RELAY_SKIP_MAX_LAG_SECS (%d) must be greater than PROFITABILITY_MAX_DELAY_SECS (%d)",
			RelaySkipMaxLagSecs, maxDelaySecs)
	}
	// Fee gas is expected BEEFY burn for static profitability; floor gas is the conservative
	// limit base. Fee above floor inverts that relationship (over-strict gate, under-padded limit).
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

// staticAggregate3FeeGas is the expected gas burn used for V2 fast-path profitability
// pricing (fiat-shamir BEEFY + messages). Uses BaseBeefyFiatShamirFeeGas (not the higher
// gas-limit floor BaseBeefyFiatShamirGas) and omits multicallBeefyGasHeadroom, which only
// pads the broadcast gas limit so nested v2_submit can forward command gas.
func staticAggregate3FeeGas(minSubmitProofGasSum uint64) uint64 {
	return BaseBeefyFiatShamirFeeGas + multicall3AggregateOverheadBuffer + minSubmitProofGasSum
}

// Multicall3 batch tx gas heuristics (nested v2_submit must satisfy Gateway v2_dispatch checks).
// gatewayDispatchOverheadBuffer must stay in sync with DISPATCH_OVERHEAD_GAS_V2 in contracts/src/Gateway.sol.
var (
	gatewayDispatchOverheadBuffer     = getEnvUint64("GATEWAY_DISPATCH_OVERHEAD_BUFFER", 24_000) // contracts/src/Gateway.sol — paired with command MaxDispatchGas in v2_dispatch
	multicall3AggregateOverheadBuffer = getEnvUint64("MULTICALL3_AGGREGATE_OVERHEAD_BUFFER", 120_000)
	// multicallBeefyGasHeadroom covers fiat-shamir BEEFY cost above BaseBeefyFiatShamirGas.
	// Without it, sequential aggregate3 (BEEFY then v2_submit) can leave too little gas for
	// Gateway's gasleft()*63/64 check and revert InsufficientGasLimit. On the fast path
	// message calls use AllowFailure=false so that reverts the whole tx; legacy batches
	// still allow failure and would otherwise land BEEFY-only.
	multicallBeefyGasHeadroom = getEnvUint64("MULTICALL_BEEFY_GAS_HEADROOM", 1_500_000)
	// multicallMaxGasLimit caps dry-run gas bumps when a strict aggregate3 sim reverts.
	multicallMaxGasLimit = getEnvUint64("MULTICALL_MAX_GAS_LIMIT", 8_000_000)
)
