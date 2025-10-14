// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package execution

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/ethereum/go-ethereum/accounts/abi"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
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
		DeliveryFeeInDot  big.Int `json:"delivery_fee_in_dot"`
		DeliveryFeeInEther big.Int `json:"delivery_fee_in_ether"`
		DryRunSuccess     bool    `json:"dry_run_success"`
		DryRunError       *string `json:"dry_run_error"`
	} `json:"asset_hub"`
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
func (g *GasEstimator) EstimateGas(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted, inboundMsg *parachain.Message, source string) (*GasEstimate, error) {
	if !g.config.Enabled {
		log.Debug("Gas estimation disabled, skipping")
		return &GasEstimate{}, nil
	}
	log.Debug("Estimating gas")

	// EventProof parameters from inboundMsg
	eventLogAddress := fmt.Sprintf("0x%x", inboundMsg.EventLog.Address[:])

	// Join topics with commas
	topicsHex := make([]string, len(inboundMsg.EventLog.Topics))
	for i, topic := range inboundMsg.EventLog.Topics {
		topicsHex[i] = fmt.Sprintf("0x%x", topic[:])
	}
	eventLogTopics := ""
	if len(topicsHex) > 0 {
		eventLogTopics = topicsHex[0]
		for i := 1; i < len(topicsHex); i++ {
			eventLogTopics += "," + topicsHex[i]
		}
	}

	eventLogData := fmt.Sprintf("0x%x", inboundMsg.EventLog.Data)

	// Encode the proof using SCALE codec
	proofEncoded, err := types.EncodeToBytes(inboundMsg.Proof)
	if err != nil {
		return nil, fmt.Errorf("failed to encode proof: %w", err)
	}
	proofHex := fmt.Sprintf("0x%x", proofEncoded)

	// Payload parameters for XCM construction and delivery fee calculation
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

	assetsHex, err := assetsToHex(ev.Payload.Assets)
	if err != nil {
		return nil, fmt.Errorf("failed to convert assets to hex: %w", err)
	}

	args := []string{
		"estimate",
		"message",
		"--event-log-address", eventLogAddress,
		"--event-log-topics", eventLogTopics,
		"--event-log-data", eventLogData,
		"--proof", proofHex,
		"--xcm-kind", fmt.Sprintf("%d", ev.Payload.Xcm.Kind),
		"--xcm-data", xcmHex,
		"--claimer", claimerHex,
		"--origin", source,
		"--value", value,
		"--execution-fee", executionFee,
		"--relayer-fee", relayerFee,
		"--assets", assetsHex,
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

// IsProfitable checks if the relayer fee is sufficient to cover the costs paid on Polkadot.
func (g *GasEstimator) IsProfitable(estimate *GasEstimate, ev *contracts.GatewayOutboundMessageAccepted) error {
	if !g.config.Enabled {
		return nil // If estimation is disabled, accept all messages
	}

	// Check if BridgeHub dry run succeeded
	if !estimate.AssetHub.DryRunSuccess {
		return fmt.Errorf("bridge hub dry run failed: %s", *estimate.AssetHub.DryRunError)
	}

	// Calculate total fee in DOT (extrinsic fee + delivery fee) - these are paid on Polkadot
	var totalFeeInDot big.Int
	totalFeeInDot.Set(&estimate.ExtrinsicFeeInDot)
	totalFeeInDot.Add(&totalFeeInDot, &estimate.AssetHub.DeliveryFeeInDot)

	maxFeeInDotInt := new(big.Int)
	if _, ok := maxFeeInDotInt.SetString(g.config.MaxGasInDot, 10); !ok {
		return fmt.Errorf("config MaxGasInDot could not be converted to Int")
	}

	// Check DOT limit
	if totalFeeInDot.Cmp(maxFeeInDotInt) == 1 {
		return fmt.Errorf("total fee in DOT exceeds limit, estimated fee %s max fee %s", totalFeeInDot.String(), g.config.MaxGasInDot)
	}

	// Calculate total fee in Ether equivalent (extrinsic fee + delivery fee converted to ETH)
	var totalFeeInEther big.Int
	totalFeeInEther.Set(&estimate.ExtrinsicFeeInEther)
	totalFeeInEther.Add(&totalFeeInEther, &estimate.AssetHub.DeliveryFeeInEther)

	maxFeeInEther := new(big.Int)
	if _, ok := maxFeeInEther.SetString(g.config.MaxGasInEther, 10); !ok {
		return fmt.Errorf("config MaxGasInEther could not be converted to Int")
	}

	// Check Ether limit
	if totalFeeInEther.Cmp(maxFeeInEther) == 1 {
		return fmt.Errorf("total fee in Ether exceeds limit, estimated fee %s max fee %s", totalFeeInEther.String(), g.config.MaxGasInEther)
	}

	// Check profitability: relayer fee (reward in ETH) must exceed total costs (in ETH equivalent)
	if ev.Payload.RelayerFee.Cmp(&totalFeeInEther) <= 0 {
		return fmt.Errorf("message is not profitable: relayer fee %s <= total fee %s", ev.Payload.RelayerFee.String(), totalFeeInEther.String())
	}

	return nil
}

// assetsToHex encodes assets as ABI-encoded hex
func assetsToHex(assets []contracts.Asset) (string, error) {
	if len(assets) == 0 {
		return "", nil
	}

	assetArrayType, err := abi.NewType("tuple[]", "Asset[]", []abi.ArgumentMarshaling{
		{Name: "kind", Type: "uint8"},
		{Name: "data", Type: "bytes"},
	})
	if err != nil {
		return "", fmt.Errorf("failed to create asset array ABI type: %w", err)
	}

	type AssetStruct struct {
		Kind uint8
		Data []byte
	}

	values := make([]AssetStruct, len(assets))
	for i, asset := range assets {
		values[i] = AssetStruct{
			Kind: asset.Kind,
			Data: asset.Data,
		}
	}

	args := abi.Arguments{{Type: assetArrayType}}
	encoded, err := args.Pack(values)
	if err != nil {
		return "", fmt.Errorf("failed to ABI encode assets: %w", err)
	}

	return "0x" + hex.EncodeToString(encoded), nil
}
