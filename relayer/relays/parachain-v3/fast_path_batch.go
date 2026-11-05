package parachainV3

import (
	"context"
	"fmt"
	"math/big"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"golang.org/x/sync/errgroup"
)

// v1FeeMode controls how V1 static fee reward is counted in TotalFeeWei.
type v1FeeMode int

const (
	// v1FeePiggyback adds V1 calls with fee=0 and feeGas=0 so they do not
	// tighten the profitability gate (V1 delivery is treated as always worth
	// including). MinGas still pads the tx gas-limit floor so nested submit
	// does not OOG. Used on instant / missed-WS (V2 pays the gate) and handover.
	v1FeePiggyback v1FeeMode = iota
	// v1FeeFull counts the normal V1 static reward + gas*price toward break-even
	// and includes V1 gas in FeeGasSum. Used for direct-relay (message-only).
	v1FeeFull
)

// buildFastPathMessageBatch scans V2 (no OFAC) and V1 in parallel, generates MMR
// proofs, and packs submit calls. V2 collected fee includes UI/service fee when
// available. V1 fee accounting follows feeMode.
func (li *BeefyInstantSyncer) buildFastPathMessageBatch(
	ctx context.Context,
	mmrAnchorRelayBlock uint64,
	rewardAddress [32]byte,
	relayGasPriceWei *big.Int,
	feeMode v1FeeMode,
	allowFailure bool,
) (*MessageBatchBuilder, int, int, error) {
	var (
		v2Tasks []*Task
		v1Tasks []*Task
	)
	eg, egCtx := errgroup.WithContext(ctx)
	eg.Go(func() error {
		tasks, err := li.beefyListener.scanner.ScanV2(egCtx, mmrAnchorRelayBlock)
		if err != nil {
			return fmt.Errorf("scan v2: %w", err)
		}
		v2Tasks = tasks
		return nil
	})
	eg.Go(func() error {
		tasks, err := li.beefyListener.scanner.ScanV1(egCtx, mmrAnchorRelayBlock)
		if err != nil {
			return fmt.Errorf("scan v1: %w", err)
		}
		v1Tasks = tasks
		return nil
	})
	if err := eg.Wait(); err != nil {
		return nil, 0, 0, err
	}

	batch := NewMessageBatchBuilder(li.ethereumWriter.GatewayAddress(), allowFailure)
	v2Count, err := li.appendV2CallsToBatch(ctx, batch, v2Tasks, mmrAnchorRelayBlock, rewardAddress)
	if err != nil {
		return nil, 0, 0, err
	}
	v1Count, err := li.appendV1CallsToBatch(ctx, batch, v1Tasks, mmrAnchorRelayBlock, relayGasPriceWei, feeMode)
	if err != nil {
		return nil, 0, 0, err
	}
	return batch, v2Count, v1Count, nil
}

func (li *BeefyInstantSyncer) appendV2CallsToBatch(
	ctx context.Context,
	batch *MessageBatchBuilder,
	tasks []*Task,
	mmrAnchorRelayBlock uint64,
	rewardAddress [32]byte,
) (int, error) {
	count := 0
	for _, task := range tasks {
		if task == nil || task.MessageProofs == nil || len(*task.MessageProofs) == 0 {
			continue
		}
		var err error
		task.ProofOutput, err = li.beefyListener.generateProof(ctx, task.ProofInput, task.Header, &mmrAnchorRelayBlock)
		if err != nil {
			return count, fmt.Errorf("generate proof for v2 task: %w", err)
		}
		for _, proof := range *task.MessageProofs {
			calldata, err := li.ethereumWriter.BuildV2SubmitCalldata(&proof, task.ProofOutput, rewardAddress)
			if err != nil {
				return count, fmt.Errorf("build v2_submit calldata for nonce %d: %w", proof.Message.OriginalMessage.Nonce, err)
			}
			batch.AddCall(
				calldata,
				minGasForV2SubmitProof(&proof),
				estimatedGasForV2SubmitProof(&proof),
				v2CollectedFeeWei(ctx, &proof, li.serviceFees),
			)
			count++
		}
	}
	return count, nil
}

func (li *BeefyInstantSyncer) appendV1CallsToBatch(
	ctx context.Context,
	batch *MessageBatchBuilder,
	tasks []*Task,
	mmrAnchorRelayBlock uint64,
	relayGasPriceWei *big.Int,
	feeMode v1FeeMode,
) (int, error) {
	if relayGasPriceWei == nil {
		relayGasPriceWei = big.NewInt(0)
	}
	count := 0
	for _, task := range tasks {
		if task == nil || task.V1MessageProofs == nil || len(*task.V1MessageProofs) == 0 {
			continue
		}
		var err error
		task.ProofOutput, err = li.beefyListener.generateProof(ctx, task.ProofInput, task.Header, &mmrAnchorRelayBlock)
		if err != nil {
			return count, fmt.Errorf("generate proof for v1 task: %w", err)
		}
		for _, proof := range *task.V1MessageProofs {
			isRelayed, err := li.beefyListener.scanner.isV1NonceRelayed(ctx, [32]byte(proof.Message.ChannelID), proof.Message.Nonce)
			if err != nil {
				return count, fmt.Errorf("re-check if v1 nonce %d is relayed: %w", proof.Message.Nonce, err)
			}
			if isRelayed {
				continue
			}
			calldata, err := li.ethereumWriter.BuildV1SubmitCalldata(&proof, task.ProofOutput)
			if err != nil {
				return count, fmt.Errorf("build submit calldata for v1 nonce %d: %w", proof.Message.Nonce, err)
			}
			v1MinGas := minGasForV1SubmitProof(&proof)
			var feeWei *big.Int
			feeGas := v1MinGas
			switch feeMode {
			case v1FeePiggyback:
				feeWei = big.NewInt(0)
				feeGas = 0 // exclude from static profitability; still pad MinGasSum
			default:
				feeWei = new(big.Int).SetUint64(BaseV1StaticRewardWei)
				v1GasFee := new(big.Int).Mul(relayGasPriceWei, new(big.Int).SetUint64(BaseV1StaticGas))
				feeWei.Add(feeWei, v1GasFee)
			}
			batch.AddCall(calldata, v1MinGas, feeGas, feeWei)
			count++
		}
	}
	return count, nil
}

// appendMessagesFromScanTasks packs V1+V2 calls from a legacy Scan result (OFAC on V2).
// V2 uses UI fee; V1 follows feeMode. Used by direct-relay and async handover.
func (li *BeefyInstantSyncer) appendMessagesFromScanTasks(
	ctx context.Context,
	batch *MessageBatchBuilder,
	tasks []*Task,
	mmrAnchorRelayBlock uint64,
	rewardAddress [32]byte,
	relayGasPriceWei *big.Int,
	feeMode v1FeeMode,
) (int, int, error) {
	v2Count := 0
	v1Count := 0
	for _, task := range tasks {
		if task == nil {
			continue
		}
		hasV2 := task.MessageProofs != nil && len(*task.MessageProofs) > 0
		hasV1 := task.V1MessageProofs != nil && len(*task.V1MessageProofs) > 0
		if !hasV2 && !hasV1 {
			continue
		}
		var err error
		task.ProofOutput, err = li.beefyListener.generateProof(ctx, task.ProofInput, task.Header, &mmrAnchorRelayBlock)
		if err != nil {
			return v2Count, v1Count, fmt.Errorf("generate proof for task: %w", err)
		}
		if hasV2 {
			for _, proof := range *task.MessageProofs {
				skipProof, err := li.checkV2ProofAlreadyRelayed(ctx, &proof)
				if err != nil {
					return v2Count, v1Count, err
				}
				if skipProof {
					continue
				}
				calldata, err := li.ethereumWriter.BuildV2SubmitCalldata(&proof, task.ProofOutput, rewardAddress)
				if err != nil {
					return v2Count, v1Count, fmt.Errorf("build v2_submit calldata for nonce %d: %w", proof.Message.OriginalMessage.Nonce, err)
				}
				batch.AddCall(
					calldata,
					minGasForV2SubmitProof(&proof),
					estimatedGasForV2SubmitProof(&proof),
					v2CollectedFeeWei(ctx, &proof, li.serviceFees),
				)
				v2Count++
			}
		}
		if hasV1 {
			n, err := li.appendV1CallsToBatch(ctx, batch, []*Task{task}, mmrAnchorRelayBlock, relayGasPriceWei, feeMode)
			if err != nil {
				return v2Count, v1Count, err
			}
			v1Count += n
		}
	}
	return v2Count, v1Count, nil
}

func (li *BeefyInstantSyncer) rewardAddressBytes() ([32]byte, error) {
	return util.HexStringTo32Bytes(li.config.RewardAddress)
}

// logBatchComposition is a small helper for consistent message-count logging.
func logBatchComposition(v2Count, v1Count int) log.Fields {
	return log.Fields{
		"v2MessageCount": v2Count,
		"v1MessageCount": v1Count,
		"messageCallCount": v2Count + v1Count,
	}
}
