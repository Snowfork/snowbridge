// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package gasestimator

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"os/exec"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/abi"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
)

// GasEstimate represents the gas estimation results from the Rust binary
type GasEstimate struct {
	// ExtrinsicFee* from `estimate message` are always 0. ethereum-v3 takes
	// Bridge Hub extrinsic cost from EstimateBatch of the real force_batch.
	ExtrinsicFeeInDot   big.Int `json:"extrinsic_fee_in_dot"`
	ExtrinsicFeeInEther big.Int `json:"extrinsic_fee_in_ether"`
	BridgeHub           struct {
		DeliveryFeeInDot   big.Int  `json:"delivery_fee_in_dot"`
		DeliveryFeeInEther big.Int  `json:"delivery_fee_in_ether"`
		TipInEther         *big.Int `json:"tip_in_ether"`
		// Mirrored Asset Hub dry-run for legacy readers; prefer AssetHubDryRun*.
		DryRunSuccess bool    `json:"dry_run_success"`
		DryRunError   *string `json:"dry_run_error"`
	} `json:"bridge_hub"`
	Beneficiaries []string `json:"beneficiaries"`
	// Ether the message deposits to a service fee recipient on Asset Hub. It is
	// Snowbridge revenue, so it counts toward the required reward, but the
	// relayer never receives it.
	ServiceFeeInEther *big.Int `json:"service_fee_in_ether"`
	// Asset Hub dry_run_xcm. Nil when the estimator binary predates the field;
	// AssetHubAccepted then falls back to the combined BridgeHub.DryRunSuccess.
	AssetHubDryRunSuccess *bool   `json:"asset_hub_dry_run_success"`
	AssetHubDryRunError   *string `json:"asset_hub_dry_run_error"`
}

// BatchDryRun is the JSON from `estimate batch`: one Utility.force_batch quote.
type BatchDryRun struct {
	ExtrinsicFeeInDot   big.Int           `json:"extrinsic_fee_in_dot"`
	ExtrinsicFeeInEther big.Int           `json:"extrinsic_fee_in_ether"`
	DryRunSuccess       bool              `json:"dry_run_success"`
	DryRunError         *string           `json:"dry_run_error"`
	Items               []BatchItemDryRun `json:"items"`
}

// BatchItemDryRun is one inner call of a force_batch dry-run, in call order.
type BatchItemDryRun struct {
	Index   uint32  `json:"index"`
	Success bool    `json:"success"`
	Error   *string `json:"error"`
}

// AssetHubAccepted reports whether the destination XCM dry-run succeeded.
// A missing field means an older binary that only reported the combined result.
func (e *GasEstimate) AssetHubAccepted() bool {
	if e == nil {
		return false
	}
	if e.AssetHubDryRunSuccess != nil {
		return *e.AssetHubDryRunSuccess
	}
	return e.BridgeHub.DryRunSuccess
}

// V2RewardWei is RelayerFee + Bridge Hub tip + Asset Hub service fee, in wei.
func (e *GasEstimate) V2RewardWei(relayerFee *big.Int) *big.Int {
	total := new(big.Int)
	if relayerFee != nil {
		total.Set(relayerFee)
	}
	if e == nil {
		return total
	}
	if e.BridgeHub.TipInEther != nil {
		total.Add(total, e.BridgeHub.TipInEther)
	}
	if e.ServiceFeeInEther != nil {
		total.Add(total, e.ServiceFeeInEther)
	}
	return total
}

// GasEstimatorConfig holds the configuration for gas estimation
type Config struct {
	// Path to the gas estimator binary
	BinaryPath string `mapstructure:"binary-path"`
	// Whether to enable gas estimation (can be disabled for testing)
	Enabled bool `mapstructure:"enabled"`
	// AssetHub web service
	AssetHubURL string `mapstructure:"asset-hub-url"`
	// BridgeHub web service
	BridgeHubURL string `mapstructure:"bridge-hub-url"`
	// Accounts that receive the dApp's service fee, as 32-byte hex. Keep in sync
	// with the dApp's NEXT_PUBLIC_SERVICE_FEE_RECIPIENT. Also used by ethereum-v3's
	// static XCM service-fee decode when estimation is disabled.
	ServiceFeeRecipients []string `mapstructure:"service-fee-recipients"`
}

func (g Config) Validate() error {
	if !g.Enabled {
		return nil
	}

	if g.BinaryPath == "" {
		return fmt.Errorf("gas estimator binary path is required when enabled")
	}

	if _, err := os.Stat(g.BinaryPath); os.IsNotExist(err) {
		return fmt.Errorf("gas estimator binary not found at path: %s", g.BinaryPath)
	}

	if g.AssetHubURL == "" {
		return fmt.Errorf("gas estimator asset-hub-url is required when enabled")
	}

	if g.BridgeHubURL == "" {
		return fmt.Errorf("gas estimator bridge-hub-url is required when enabled")
	}

	return nil
}

// GasEstimator provides gas estimation functionality
type Estimator struct {
	config Config
}

// New creates a gas estimator instance.
func New(config Config) *Estimator {
	return &Estimator{config: config}
}

// Enabled reports whether dry-run / profitability checks are active.
func (g *Estimator) Enabled() bool {
	return g != nil && g.config.Enabled
}

// EstimateGas prices Asset Hub delivery / dry-run, tip, and service fee for a
// message. It does not quote Bridge Hub extrinsic cost (ExtrinsicFee* stay 0)
// and does not dry-run BH submit; ethereum-v3 gets both from EstimateBatch of
// the force_batch it will actually submit (optionally with a beacon update).
func (g *Estimator) EstimateGas(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted, inboundMsg *parachain.MessageV3, source string, relayerPublicKey string) (*GasEstimate, error) {
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
	eventLogTxIndex := fmt.Sprintf("%d", inboundMsg.EventLog.TxIndex)

	inboundV2Msg, err := parachain.ConvertToV2Message(inboundMsg)
	if err != nil {
		return nil, fmt.Errorf("failed to convert to v2 message: %w", err)
	}

	proofEncoded, err := types.EncodeToBytes(&inboundV2Msg.Proof)
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
		"--asset-hub-url", g.config.AssetHubURL,
		"--bridge-hub-url", g.config.BridgeHubURL,
		"estimate",
		"message",
		"--event-log-address", eventLogAddress,
		"--event-log-topics", eventLogTopics,
		"--event-log-data", eventLogData,
		"--event-log-tx-index", eventLogTxIndex,
		"--proof", proofHex,
		"--xcm-kind", fmt.Sprintf("%d", ev.Payload.Xcm.Kind),
		"--xcm-data", xcmHex,
		"--claimer", claimerHex,
		"--origin", source,
		"--value", value,
		"--execution-fee", executionFee,
		"--relayer-fee", relayerFee,
		"--assets", assetsHex,
		"--relayer-account", relayerPublicKey,
		"--nonce", fmt.Sprintf("%d", ev.Nonce),
		"--service-fee-recipients", strings.Join(g.config.ServiceFeeRecipients, ","),
	}

	log.WithFields(log.Fields{
		"binary": g.config.BinaryPath,
		"args":   args,
	}).Info("executing gas estimation with args")

	output, err := g.run(ctx, args)
	if err != nil {
		return nil, err
	}

	var estimate GasEstimate
	if err := json.Unmarshal(output, &estimate); err != nil {
		return nil, fmt.Errorf("failed to parse gas estimation response: %w", err)
	}

	logFields := log.Fields{
		"extrinsic_fee_dot":   estimate.ExtrinsicFeeInDot.String(),
		"extrinsic_fee_ether": estimate.ExtrinsicFeeInEther.String(),
		"delivery_fee_dot":    estimate.BridgeHub.DeliveryFeeInDot.String(),
		"delivery_fee_ether":  estimate.BridgeHub.DeliveryFeeInEther.String(),
		"dry_run_success":     estimate.BridgeHub.DryRunSuccess,
	}
	if estimate.BridgeHub.TipInEther != nil {
		logFields["tip_in_ether"] = estimate.BridgeHub.TipInEther.String()
	}
	log.WithFields(logFields).Debug("gas estimation completed")

	return &estimate, nil
}

// EstimateBatch dry-runs Utility.force_batch of the given SCALE RuntimeCalls
// (0x-prefixed hex) and returns the single extrinsic fee plus per-item results.
// calls are the same bytes ForceBatch would submit, in that order.
func (g *Estimator) EstimateBatch(ctx context.Context, calls []string, relayerAccount string) (*BatchDryRun, error) {
	if !g.config.Enabled {
		return &BatchDryRun{DryRunSuccess: true}, nil
	}
	if len(calls) == 0 {
		return nil, fmt.Errorf("estimate batch: no calls")
	}

	args := []string{
		"--asset-hub-url", g.config.AssetHubURL,
		"--bridge-hub-url", g.config.BridgeHubURL,
		"estimate",
		"batch",
		"--relayer-account", relayerAccount,
	}
	for _, call := range calls {
		args = append(args, "--call", call)
	}

	log.WithFields(log.Fields{
		"binary":    g.config.BinaryPath,
		"callCount": len(calls),
	}).Info("executing force_batch gas estimation")

	output, err := g.run(ctx, args)
	if err != nil {
		return nil, err
	}

	var dry BatchDryRun
	if err := json.Unmarshal(output, &dry); err != nil {
		return nil, fmt.Errorf("failed to parse batch dry-run response: %w", err)
	}
	log.WithFields(log.Fields{
		"extrinsic_fee_dot":   dry.ExtrinsicFeeInDot.String(),
		"extrinsic_fee_ether": dry.ExtrinsicFeeInEther.String(),
		"dry_run_success":     dry.DryRunSuccess,
		"items":               len(dry.Items),
	}).Debug("force_batch gas estimation completed")
	return &dry, nil
}

func (g *Estimator) run(ctx context.Context, args []string) ([]byte, error) {
	cmd := exec.CommandContext(ctx, g.config.BinaryPath, args...)
	output, err := cmd.Output()
	if err != nil {
		var stderr string
		if exitError, ok := err.(*exec.ExitError); ok {
			stderr = string(exitError.Stderr)
		}
		log.WithFields(log.Fields{
			"stdout": string(output),
			"stderr": stderr,
			"error":  err.Error(),
		}).Error("gas estimator execution failed")
		execErr := fmt.Errorf("gas estimator execution failed: %w: %s", err, stderr)
		if error_tracking.IsStaleMetadataError(execErr) {
			log.WithFields(log.Fields{
				"binary": g.config.BinaryPath,
				"stderr": stderr,
			}).Error("ALERT: gas estimator metadata incompatible with live runtime - rebuild and redeploy the relayer image (release-relayer)")
		}
		return nil, execErr
	}
	return output, nil
}

// IsProfitable is a legacy single-message break-even helper (tests / older
// callers). ethereum-v3 uses checkV2Profitable + selectProfitableBatch instead.
// ExtrinsicFeeInEther from estimate message is 0; pass a batch fee explicitly
// in tests if you need BH extrinsic cost in the comparison.
func (g *Estimator) IsProfitable(ctx context.Context, estimate *GasEstimate, ev *contracts.GatewayOutboundMessageAccepted, currentBlock uint64) error {
	if !g.config.Enabled {
		return nil // If estimation is disabled, accept all messages
	}

	// estimate message dry-runs the Asset Hub destination XCM (mirrored into
	// BridgeHub.DryRunSuccess for this struct). Bridge Hub submit is not checked here.
	if !estimate.AssetHubAccepted() {
		errMsg := ""
		if estimate.AssetHubDryRunError != nil {
			errMsg = *estimate.AssetHubDryRunError
		} else if estimate.BridgeHub.DryRunError != nil {
			errMsg = *estimate.BridgeHub.DryRunError
		}
		return fmt.Errorf("asset hub dry run failed: %s", errMsg)
	}

	// delivery fee + optional ExtrinsicFeeInEther (0 from estimate message; tests may set it)
	var totalFeeInEther big.Int
	totalFeeInEther.Set(&estimate.ExtrinsicFeeInEther)
	totalFeeInEther.Add(&totalFeeInEther, &estimate.BridgeHub.DeliveryFeeInEther)

	// Gas break-even: relayer fee + tip + UI/service fee must exceed total costs.
	// Service fee is Snowbridge revenue the relayer does not receive, but it still
	// counts toward the break-even gate (volume×rewardRate was removed).
	var totalReward big.Int
	totalReward.Set(ev.Payload.RelayerFee)
	if estimate.BridgeHub.TipInEther != nil {
		totalReward.Add(&totalReward, estimate.BridgeHub.TipInEther)
	}
	if estimate.ServiceFeeInEther != nil {
		totalReward.Add(&totalReward, estimate.ServiceFeeInEther)
	}

	if totalReward.Cmp(&totalFeeInEther) <= 0 {
		return fmt.Errorf("message is not profitable: total reward %s <= total fee %s", totalReward.String(), totalFeeInEther.String())
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
