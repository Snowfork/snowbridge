package parachainV3

import (
	"math/big"
	"testing"
)

func TestHandoverTipLag(t *testing.T) {
	if got := handoverTipLag(100, 100); got != 0 {
		t.Fatalf("at tip: got %d want 0", got)
	}
	if got := handoverTipLag(90, 100); got != 0 {
		t.Fatalf("tip behind LC: got %d want 0", got)
	}
	// Tip−LC grows with the relay tip (unlike commitment−LC, which plateaus in
	// the ~2500-block session window during a validator-set handover).
	if got := handoverTipLag(100+2500, 100); got != 2500 {
		t.Fatalf("session-window tip lag: got %d want 2500", got)
	}
	if got := handoverTipLag(100+4801, 100); got != 4801 {
		t.Fatalf("past skip window tip lag: got %d want 4801", got)
	}
}

func TestShouldSkipHandoverOnHighGas(t *testing.T) {
	const (
		highGas = 250_000_000 // 0.25 gwei
		maxLag  = 4800        // RELAY_SKIP_MAX_LAG_SECS / 6s
	)
	above := big.NewInt(450_000_000) // ~0.45 gwei
	below := big.NewInt(200_000_000) // ~0.20 gwei

	if !shouldSkipHandoverOnHighGas(above, 2500, highGas, maxLag) {
		t.Fatal("tip lag 2500 + high gas: expected skip")
	}
	// lag <= maxLag still skips; force only when lag > maxLag.
	if !shouldSkipHandoverOnHighGas(above, 4800, highGas, maxLag) {
		t.Fatal("tip lag == maxLag + high gas: expected skip")
	}
	if shouldSkipHandoverOnHighGas(above, 4801, highGas, maxLag) {
		t.Fatal("tip lag 4801 + high gas: expected force (do not skip)")
	}
	if shouldSkipHandoverOnHighGas(below, 5000, highGas, maxLag) {
		t.Fatal("gas at/under threshold: never skip on gas gate")
	}
	if shouldSkipHandoverOnHighGas(nil, 5000, highGas, maxLag) {
		t.Fatal("nil gas: expected no skip")
	}
}
