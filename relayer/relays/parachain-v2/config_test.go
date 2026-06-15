package parachain

import (
	"testing"
)

func TestFeeConfigValidate(t *testing.T) {
	tests := []struct {
		name        string
		config      FeeConfig
		expectError bool
	}{
		{
			name: "valid config with 1:1 ratio",
			config: FeeConfig{
				BaseDeliveryGas:     100000,
				BaseUnlockGas:       60000,
				BaseMintGas:         60000,
				FeeRatioNumerator:   1,
				FeeRatioDenominator: 1,
			},
			expectError: false,
		},
		{
			name: "valid config with 5:4 ratio",
			config: FeeConfig{
				BaseDeliveryGas:     100000,
				BaseUnlockGas:       60000,
				BaseMintGas:         60000,
				FeeRatioNumerator:   5,
				FeeRatioDenominator: 4,
			},
			expectError: false,
		},
		{
			name: "invalid config with zero denominator",
			config: FeeConfig{
				BaseDeliveryGas:     100000,
				BaseUnlockGas:       60000,
				BaseMintGas:         60000,
				FeeRatioNumerator:   1,
				FeeRatioDenominator: 0,
			},
			expectError: true,
		},
		{
			name: "invalid config with zero numerator",
			config: FeeConfig{
				BaseDeliveryGas:     100000,
				BaseUnlockGas:       60000,
				BaseMintGas:         60000,
				FeeRatioNumerator:   0,
				FeeRatioDenominator: 1,
			},
			expectError: true,
		},
		{
			name: "invalid config with both zero",
			config: FeeConfig{
				BaseDeliveryGas:     100000,
				BaseUnlockGas:       60000,
				BaseMintGas:         60000,
				FeeRatioNumerator:   0,
				FeeRatioDenominator: 0,
			},
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if tt.expectError && err == nil {
				t.Errorf("expected error but got none")
			}
			if !tt.expectError && err != nil {
				t.Errorf("expected no error but got: %v", err)
			}
		})
	}
}
