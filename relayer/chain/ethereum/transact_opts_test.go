package ethereum_test

import (
	"math/big"
	"testing"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/config"
	"github.com/stretchr/testify/assert"
)

func TestCalculateFee_NoFeesSpecified(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: 0,
		GasTipMultiplier: 0,
	}

	var suggestedFee *big.Int
	var suggestedTip *big.Int

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Nil(t, fee.GasFeeCap)
	assert.Nil(t, fee.GasTipCap)
}

func TestCalculateFee_DynamicFeeTipOnly(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: 0,
		GasTipMultiplier: 1.5,
	}

	var suggestedFee *big.Int
	var suggestedTip *big.Int = big.NewInt(10_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Nil(t, fee.GasFeeCap)
	assert.Equal(t, fee.GasTipCap, big.NewInt(15_000))
}

func TestCalculateFee_DynamicFeesBaseOnly(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: 3.5,
		GasTipMultiplier: 0,
	}

	var suggestedFee *big.Int = big.NewInt(10_000)
	var suggestedTip *big.Int

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(35_000))
	assert.Nil(t, fee.GasTipCap)
}

func TestCalculateFee_DynamicFees(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: 3.5,
		GasTipMultiplier: 1.5,
	}

	var suggestedFee *big.Int = big.NewInt(10_000)
	var suggestedTip *big.Int = big.NewInt(10_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(35_000))
	assert.Equal(t, fee.GasTipCap, big.NewInt(15_000))
}

func TestCalculateFee_FixedFeeTipOnly(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        1000,
		GasFeeMultiplier: 0,
		GasTipMultiplier: 0,
	}

	var suggestedFee *big.Int
	var suggestedTip *big.Int

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Nil(t, fee.GasFeeCap)
	assert.Equal(t, fee.GasTipCap, big.NewInt(1000))
}

func TestCalculateFee_FixedFeeBaseOnly(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        2000,
		GasTipCap:        0,
		GasFeeMultiplier: 0,
		GasTipMultiplier: 0,
	}

	var suggestedFee *big.Int
	var suggestedTip *big.Int

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(2000))
	assert.Nil(t, fee.GasTipCap)
}

func TestCalculateFee_FixedFees(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        2000,
		GasTipCap:        1000,
		GasFeeMultiplier: 0,
		GasTipMultiplier: 0,
	}

	var suggestedFee *big.Int
	var suggestedTip *big.Int

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(2000))
	assert.Equal(t, fee.GasTipCap, big.NewInt(1000))
}

func TestCalculateFee_FixedFeeHasNoDefaultMaximum(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        ethereum.DefaultMaxGas + 1_000,
		GasTipCap:        ethereum.DefaultMaxTip + 1_000,
		GasFeeMultiplier: 0,
		GasTipMultiplier: 0,
	}

	var suggestedFee *big.Int
	var suggestedTip *big.Int

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(ethereum.DefaultMaxGas+1_000))
	assert.Equal(t, fee.GasTipCap, big.NewInt(ethereum.DefaultMaxTip+1_000))
}

func TestCalculateFee_DynamicFeeAppliesMaximum(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: 1000,
		GasTipMultiplier: 200,
	}

	var suggestedFee *big.Int = big.NewInt(4_000_000_000)
	var suggestedTip *big.Int = big.NewInt(1_000_000_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(ethereum.DefaultMaxGas))
	assert.Equal(t, fee.GasTipCap, big.NewInt(ethereum.DefaultMaxTip))
}

func TestCalculateFee_DynamicFeeWithMaximumOverride(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        400_000_000_000,
		GasTipCap:        100_000_000_000,
		GasFeeMultiplier: 1000,
		GasTipMultiplier: 200,
	}

	var suggestedFee *big.Int = big.NewInt(4_000_000_000)
	var suggestedTip *big.Int = big.NewInt(1_000_000_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(400_000_000_000))
	assert.Equal(t, fee.GasTipCap, big.NewInt(100_000_000_000))
}

func TestCalculateFee_DynamicFee(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        400_000_000_000,
		GasTipCap:        100_000_000_000,
		GasFeeMultiplier: 100,
		GasTipMultiplier: 85,
	}

	var suggestedFee *big.Int = big.NewInt(3_500_000_000)
	var suggestedTip *big.Int = big.NewInt(1_000_000_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(350_000_000_000))
	assert.Equal(t, fee.GasTipCap, big.NewInt(85_000_000_000))
}

func TestCalculateFee_DynamicFeeDoesNotYieldLessThanSuggested(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: 0.4,
		GasTipMultiplier: 0.9,
	}

	var suggestedFee *big.Int = big.NewInt(4_000_000_000)
	var suggestedTip *big.Int = big.NewInt(1_000_000_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(4_000_000_000))
	assert.Equal(t, fee.GasTipCap, big.NewInt(1_000_000_000))
}

func TestCalculateFee_DynamicFeeDoesNotYieldNegative(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: -1.4,
		GasTipMultiplier: -4.9,
	}

	var suggestedFee *big.Int = big.NewInt(4_000_000_000)
	var suggestedTip *big.Int = big.NewInt(1_000_000_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Equal(t, fee.GasFeeCap, big.NewInt(4_000_000_000))
	assert.Equal(t, fee.GasTipCap, big.NewInt(1_000_000_000))
}

func TestCalculateFee_DynamicFeeIgnoresSuggestionsIfNoMultipliers(t *testing.T) {
	config := config.EthereumConfig{
		GasFeeCap:        0,
		GasTipCap:        0,
		GasFeeMultiplier: 0,
		GasTipMultiplier: 0,
	}

	var suggestedFee *big.Int = big.NewInt(4_000_000_000)
	var suggestedTip *big.Int = big.NewInt(1_000_000_000)

	fee := ethereum.CalculateFee(config, suggestedFee, suggestedTip)

	assert.Nil(t, fee.GasFeeCap)
	assert.Nil(t, fee.GasTipCap)
}
