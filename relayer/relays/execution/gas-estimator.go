// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package execution

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"math/big"
	"os"
	"os/exec"
)

// GasEstimate represents the gas estimation results from the Rust binary
type GasEstimate struct {
	ExtrinsicFeeInDot   big.Int `json:"extrinsic_fee_in_dot"`
	ExtrinsicFeeInEther big.Int `json:"extrinsic_fee_in_ether"`
	AssetHub            struct {
		ExecutionFeeInDot   big.Int `json:"execution_fee_in_dot"`
		ExecutionFeeInEther big.Int `json:"execution_fee_in_ether"`
		DeliveryFeeInDot    big.Int `json:"delivery_fee_in_dot"`
		DeliveryFeeInEther  big.Int `json:"delivery_fee_in_ether"`
		DryRunSuccess       bool    `json:"dry_run_success"`
		DryRunError         *string `json:"dry_run_error"`
	} `json:"asset_hub"`
	Destination struct {
		ExecutionFeeInDot   *big.Int `json:"execution_fee_in_dot"`
		ExecutionFeeInEther *big.Int `json:"execution_fee_in_ether"`
		DeliveryFeeInDot    *big.Int `json:"delivery_fee_in_dot"`
		DeliveryFeeInEther  *big.Int `json:"delivery_fee_in_ether"`
		DryRunSuccess       *bool    `json:"dry_run_success"`
		DryRunError         *string  `json:"dry_run_error"`
		ParaID              *uint32  `json:"para_id"`
	} `json:"destination"`
}

// GasEstimatorConfig holds the configuration for gas estimation
type GasEstimatorConfig struct {
	// Path to the gas estimator binary
	BinaryPath string `mapstructure:"binary-path"`
	// Maximum acceptable gas in DOT (10^10 planck = 1 DOT)
	MaxGasInDot string `mapstructure:"max-gas-in-dot"`
	// Maximum acceptable gas in Ether (wei)
	MaxGasInEther string `mapstructure:"max-gas-in-ether"`
	// Whether to enable gas estimation (can be disabled for testing)
	Enabled bool `mapstructure:"enabled"`
	// Environment variables to pass to the binary (like WS URLs)
	Environment map[string]string `mapstructure:"environment"`
	// Environment name for gas estimator (polkadot_mainnet, westend_sepolia)
	EstimatorEnvironment string `mapstructure:"estimator-environment"`
}

func (g GasEstimatorConfig) Validate() error {
	if !g.Enabled {
		return nil
	}

	if g.BinaryPath == "" {
		return fmt.Errorf("gas estimator binary path is required when enabled")
	}

	if _, err := os.Stat(g.BinaryPath); os.IsNotExist(err) {
		return fmt.Errorf("gas estimator binary not found at path: %s", g.BinaryPath)
	}

	if g.EstimatorEnvironment != "" && g.EstimatorEnvironment != "polkadot_mainnet" && g.EstimatorEnvironment != "westend_sepolia" {
		return fmt.Errorf("invalid estimator environment: %s. Must be 'polkadot_mainnet' or 'westend_sepolia'", g.EstimatorEnvironment)
	}

	return nil
}

// GasEstimator provides gas estimation functionality
type GasEstimator struct {
	config GasEstimatorConfig
}

// NewGasEstimator creates a new gas estimator instance
func NewGasEstimator(config GasEstimatorConfig) *GasEstimator {
	return &GasEstimator{
		config: config,
	}
}

// EstimateGas estimates the gas cost for processing a message
func (g *GasEstimator) EstimateGas(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted, source string) (*GasEstimate, error) {
	if !g.config.Enabled {
		log.Debug("Gas estimation disabled, skipping")
		return &GasEstimate{}, nil
	}
	log.Debug("Estimating gas")

	xcmHex := ""
	if len(ev.Payload.Xcm.Data) > 0 {
		xcmHex = fmt.Sprintf("0x%x", ev.Payload.Xcm.Data)
	}

	claimerHex := ""
	if len(ev.Payload.Claimer) > 0 {
		claimerHex = fmt.Sprintf("0x%x", ev.Payload.Claimer)
	}

	value := ev.Payload.Value.String()
	executionFee := ev.Payload.ExecutionFee.String()
	relayerFee := ev.Payload.RelayerFee.String()

	// Convert assets to JSON format expected by the gas estimator
	assetsJSON, err := assetsToJSON(ev.Payload.Assets)
	if err != nil {
		return nil, fmt.Errorf("failed to convert assets to JSON: %w", err)
	}

	args := []string{
		"estimate",
		"message",
		"--xcm-kind", fmt.Sprintf("%d", ev.Payload.Xcm.Kind),
		"--xcm-data", xcmHex,
		"--claimer", claimerHex,
		"--origin", source,
		"--value", value,
		"--execution-fee", executionFee,
		"--relayer-fee", relayerFee,
		"--assets", assetsJSON,
	}

	cmd := exec.CommandContext(ctx, g.config.BinaryPath, args...)

	if g.config.Environment != nil {
		env := make([]string, 0, len(g.config.Environment))
		for key, value := range g.config.Environment {
			env = append(env, fmt.Sprintf("%s=%s", key, value))
		}
		cmd.Env = env
	}

	log.WithFields(log.Fields{
		"binary": g.config.BinaryPath,
		"args":   args,
	}).Info("executing gas estimation with args")

	// Execute the command and capture both stdout and stderr
	output, err := cmd.Output()
	if err != nil {
		// Get stderr output if available
		var stderr string
		if exitError, ok := err.(*exec.ExitError); ok {
			stderr = string(exitError.Stderr)
		}

		log.WithFields(log.Fields{
			"stdout": string(output),
			"stderr": stderr,
			"error":  err.Error(),
		}).Error("gas estimator execution failed")

		return nil, fmt.Errorf("gas estimator execution failed: %w", err)
	}

	var estimate GasEstimate
	if err := json.Unmarshal(output, &estimate); err != nil {
		return nil, fmt.Errorf("failed to parse gas estimation response: %w", err)
	}

	log.WithFields(log.Fields{
		"estimate": estimate,
	}).Debug("gas estimation completed")

	return &estimate, nil
}

// IsProfitable checks if the ether provided with the message is sufficient to cover costs.
func (g *GasEstimator) IsProfitable(estimate *GasEstimate, ev *contracts.GatewayOutboundMessageAccepted) error {
	if !g.config.Enabled {
		return nil // If estimation is disabled, accept all messages
	}

	if !estimate.AssetHub.DryRunSuccess {
		return fmt.Errorf("asset hub dry run failed: %s", *estimate.AssetHub.DryRunError)
	}

	if estimate.Destination.DryRunSuccess != nil && !*estimate.Destination.DryRunSuccess {
		return fmt.Errorf("destination hub dry run failed: %s", *estimate.Destination.DryRunError)
	}

	var totalGasInDot big.Int
	totalGasInDot.Set(&estimate.ExtrinsicFeeInDot)

	totalGasInDot.Add(&totalGasInDot, &estimate.AssetHub.ExecutionFeeInDot)
	totalGasInDot.Add(&totalGasInDot, &estimate.AssetHub.DeliveryFeeInDot)

	if estimate.Destination.ExecutionFeeInDot != nil {
		totalGasInDot.Add(&totalGasInDot, estimate.Destination.ExecutionFeeInDot)
	}

	if estimate.Destination.DeliveryFeeInDot != nil {
		totalGasInDot.Add(&totalGasInDot, estimate.Destination.DeliveryFeeInDot)
	}

	maxGasInDotInt := new(big.Int)
	if _, ok := maxGasInDotInt.SetString(g.config.MaxGasInDot, 10); !ok {
		return fmt.Errorf("config MaxGasInDot could not be converted to Int")
	}

	// Check DOT limit
	if totalGasInDot.Cmp(maxGasInDotInt) == 1 {
		return fmt.Errorf("gas cost in DOT exceeds limit, estimated gas cost %s max gas dot %s", totalGasInDot.String(), g.config.MaxGasInDot)
	}

	var totalGasInEther big.Int
	totalGasInEther.Set(&estimate.ExtrinsicFeeInEther)
	totalGasInEther.Add(&totalGasInEther, &estimate.AssetHub.ExecutionFeeInEther)
	totalGasInEther.Add(&totalGasInEther, &estimate.AssetHub.DeliveryFeeInEther)

	if estimate.Destination.ExecutionFeeInEther != nil {
		totalGasInEther.Add(&totalGasInEther, estimate.Destination.ExecutionFeeInEther)
	}

	if estimate.Destination.DeliveryFeeInEther != nil {
		totalGasInEther.Add(&totalGasInEther, estimate.Destination.DeliveryFeeInEther)
	}

	MaxGasInEther := new(big.Int)
	if _, ok := MaxGasInEther.SetString(g.config.MaxGasInEther, 10); !ok {
		return fmt.Errorf("config MaxGasInEther could not be converted to Int")
	}

	// Check Ether limit
	if totalGasInEther.Cmp(MaxGasInEther) == 1 {
		return fmt.Errorf("gas cost in Ether exceeds limit, estimated gas cost %s max gas dot %s", totalGasInEther.String(), g.config.MaxGasInEther)
	}

	// Check if AssetHub execution fee actually covers expected AssetHub execution
	if estimate.AssetHub.ExecutionFeeInEther.Cmp(ev.Payload.ExecutionFee) == 1 {
		return fmt.Errorf("asset hub execution fee does not cover estimated execution cost, estimated asset hub execution %s provided asset hub execution %s", estimate.AssetHub.ExecutionFeeInEther.String(), ev.Payload.ExecutionFee.String())
	}

	var profitability big.Int
	profitability.Set(&estimate.ExtrinsicFeeInEther)

	profitability.Add(&profitability, &estimate.AssetHub.DeliveryFeeInEther)
	profitability.Add(&profitability, &estimate.AssetHub.ExecutionFeeInEther)
	profitability.Add(&profitability, ev.Payload.RelayerFee)

	if profitability.Cmp(ev.Payload.Value) == 1 {
		return fmt.Errorf("ether value provided not profitable to relay, profitability %s messageValue %s", profitability.String(), ev.Payload.Value.String())
	}

	return nil
}

// BridgeAssetJSON represents the JSON structure expected by the Rust gas estimator
type BridgeAssetJSON struct {
	Kind      string `json:"kind"`
	Token     string `json:"token,omitempty"`      // For native tokens
	Amount    string `json:"amount"`               // Always present
	ForeignID string `json:"foreign_id,omitempty"` // For foreign tokens
}

// AsNativeTokenERC20 represents the ABI structure for native ERC20 assets (kind = 0)
type AsNativeTokenERC20 struct {
	Token  common.Address
	Amount *big.Int
}

// AsForeignTokenERC20 represents the ABI structure for foreign ERC20 assets (kind = 1)
type AsForeignTokenERC20 struct {
	ForeignID [32]byte
	Amount    *big.Int
}

const (
	AssetKindNativeTokenERC20  = 0
	AssetKindForeignTokenERC20 = 1
)

func assetsToJSON(assets []contracts.Asset) (string, error) {
	bridgeAssets := make([]BridgeAssetJSON, 0, len(assets))

	// Define ABI types for decoding asset data
	nativeTokenType, err := abi.NewType("tuple", "AsNativeTokenERC20", []abi.ArgumentMarshaling{
		{Name: "token", Type: "address"},
		{Name: "amount", Type: "uint128"},
	})
	if err != nil {
		return "", fmt.Errorf("failed to create native token ABI type: %w", err)
	}

	foreignTokenType, err := abi.NewType("tuple", "AsForeignTokenERC20", []abi.ArgumentMarshaling{
		{Name: "foreignID", Type: "bytes32"},
		{Name: "amount", Type: "uint128"},
	})
	if err != nil {
		return "", fmt.Errorf("failed to create foreign token ABI type: %w", err)
	}

	nativeArgs := abi.Arguments{{Type: nativeTokenType}}
	foreignArgs := abi.Arguments{{Type: foreignTokenType}}

	for i, asset := range assets {
		switch asset.Kind {
		case AssetKindNativeTokenERC20:
			// Decode native token data
			values, err := nativeArgs.Unpack(asset.Data)
			if err != nil {
				return "", fmt.Errorf("failed to decode native token asset %d: %w", i, err)
			}

			if len(values) != 1 {
				return "", fmt.Errorf("expected 1 value from native token unpack, got %d", len(values))
			}

			// The unpacked value is a struct with Token and Amount fields
			nativeStruct := values[0].(struct {
				Token  common.Address `json:"token"`
				Amount *big.Int       `json:"amount"`
			})

			bridgeAssets = append(bridgeAssets, BridgeAssetJSON{
				Kind:   "native",
				Token:  nativeStruct.Token.Hex(),
				Amount: nativeStruct.Amount.String(),
			})

		case AssetKindForeignTokenERC20:
			// Decode foreign token data
			values, err := foreignArgs.Unpack(asset.Data)
			if err != nil {
				return "", fmt.Errorf("failed to decode foreign token asset %d: %w", i, err)
			}

			if len(values) != 1 {
				return "", fmt.Errorf("expected 1 value from foreign token unpack, got %d", len(values))
			}

			// The unpacked value is a struct with ForeignID and Amount fields
			foreignStruct := values[0].(struct {
				ForeignID [32]byte `json:"foreignID"`
				Amount    *big.Int `json:"amount"`
			})

			bridgeAssets = append(bridgeAssets, BridgeAssetJSON{
				Kind:      "foreign",
				ForeignID: fmt.Sprintf("0x%x", foreignStruct.ForeignID),
				Amount:    foreignStruct.Amount.String(),
			})

		default:
			return "", fmt.Errorf("unknown asset kind %d for asset %d", asset.Kind, i)
		}
	}

	jsonBytes, err := json.Marshal(bridgeAssets)
	if err != nil {
		return "", fmt.Errorf("failed to marshal assets to JSON: %w", err)
	}

	return string(jsonBytes), nil
}
