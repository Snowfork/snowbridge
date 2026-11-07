package parachainV3

import "math/big"

// handoverTipLag is how far the on-chain BeefyClient lags the relay BEEFY tip.
// Use tip−LC (not commitment−LC): during validator-set handover the proveable
// commitment sits in a ~session-length window ahead of the LC, so commitment lag
// plateaus (~2500 blocks) and never trips RelaySkipMaxLagBlocks (~4800 / 8h).
func handoverTipLag(relayBeefyTip, lightClientBeefyBlock uint64) uint64 {
	if relayBeefyTip > lightClientBeefyBlock {
		return relayBeefyTip - lightClientBeefyBlock
	}
	return 0
}

// shouldSkipHandoverOnHighGas is the HIGH_GAS park gate for async handover.
// Past maxLagBlocks tip lag we force the sync even when gas is above threshold.
func shouldSkipHandoverOnHighGas(gasWei *big.Int, tipLag, highGasThresholdWei, maxLagBlocks uint64) bool {
	if gasWei == nil {
		return false
	}
	if gasWei.Cmp(new(big.Int).SetUint64(highGasThresholdWei)) <= 0 {
		return false
	}
	return tipLag <= maxLagBlocks
}
