package parachainV3

import "testing"

func TestValidateProfitabilityConfigFeeGasVsFloor(t *testing.T) {
	origFee, origFloor := BaseBeefyFiatShamirFeeGas, BaseBeefyFiatShamirGas
	t.Cleanup(func() {
		BaseBeefyFiatShamirFeeGas, BaseBeefyFiatShamirGas = origFee, origFloor
	})

	// Defaults / fee ≤ floor: should still pass other invariants in a clean test env.
	BaseBeefyFiatShamirFeeGas = 1_850_000
	BaseBeefyFiatShamirGas = 2_000_000
	if err := validateProfitabilityConfig(); err != nil {
		t.Fatalf("fee ≤ floor: unexpected error: %v", err)
	}

	BaseBeefyFiatShamirFeeGas = 2_000_000
	BaseBeefyFiatShamirGas = 2_000_000
	if err := validateProfitabilityConfig(); err != nil {
		t.Fatalf("fee == floor: unexpected error: %v", err)
	}

	BaseBeefyFiatShamirFeeGas = 2_100_000
	BaseBeefyFiatShamirGas = 2_000_000
	if err := validateProfitabilityConfig(); err == nil {
		t.Fatal("fee > floor: expected error")
	}
}
