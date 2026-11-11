package parachainV3

import (
	"context"
	"fmt"
	"math/big"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
	"golang.org/x/sync/errgroup"
)

// doCatchUp is the single-service catch-up tick: direct-relay already-anchored
// messages, then missed-WS fiat-shamir+messages, then kick async handover when
// a mandatory commitment remains. Serialized with the WS instant path via li.mu;
// async handover continues outside the lock after kick.
func (li *BeefyInstantSyncer) doCatchUp(ctx context.Context) error {
	li.mu.Lock()
	defer li.mu.Unlock()

	latestBeefyBlockOnEthereum, _, err := li.beefyListener.fetchLatestBeefyBlock(ctx)
	if err != nil {
		return fmt.Errorf("fetch latest beefy block from light client: %w", err)
	}

	relayed, err := li.tryDirectRelayAnchoredMessages(ctx, latestBeefyBlockOnEthereum)
	if err != nil {
		log.WithError(err).Warn("Failed to run direct relay check for anchored messages; continuing catch-up")
		if error_tracking.IsTransientError(err) {
			li.reconnectAndRebindEthereum(ctx)
		}
	} else if relayed {
		return nil
	}

	finalizedBeefyBlockHash, err := li.beefyListener.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
	if err != nil {
		return fmt.Errorf("fetch beefy finalized head: %w", err)
	}
	finalizedBeefyBlockHeader, err := li.beefyListener.relaychainConn.API().RPC.Chain.GetHeader(finalizedBeefyBlockHash)
	if err != nil {
		return fmt.Errorf("fetch block header: %w", err)
	}
	beefyBlockNumber := uint64(finalizedBeefyBlockHeader.Number)

	consensusResult, err := li.beefyOnDemandRelay.BuildConsensusTask(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("build beefy consensus task: %w", err)
	}
	if consensusResult == nil {
		log.WithField("beefyBlockNumber", beefyBlockNumber).Debug("[catch-up] No beefy consensus update needed")
		return nil
	}

	// Missed-WS retry: same profitable fiat-shamir + messages path as instant
	// (>=1 V2 pays the gate; V1 piggybacks fee=0). V1-only waits for direct-relay
	// or handover.
	submitted, err := li.tryMissedWSFiatShamir(ctx, consensusResult)
	if err != nil {
		return err
	}
	if submitted {
		return nil
	}

	if !consensusResult.IsHandover {
		log.WithFields(log.Fields{
			"beefyBlockNumber":     beefyBlockNumber,
			"mmrAnchorRelayBlock":  consensusResult.MMRAnchorRelayBlock,
			"isHandoverCommitment": false,
		}).Debug("[catch-up] No profitable message path and no handover; skipping")
		return nil
	}

	li.kickAsyncHandover(ctx, consensusResult)
	return nil
}

// tryMissedWSFiatShamir attempts fiat-shamir + pending messages for a tip the WS
// path may have missed. Same rule as instant: >=1 V2 pays the gate; V1 rides as
// piggyback (fee=0 / feeGas=0). V1-only is left to direct-relay or handover.
func (li *BeefyInstantSyncer) tryMissedWSFiatShamir(ctx context.Context, consensusResult *beefy.ConsensusTaskResult) (bool, error) {
	if consensusResult == nil || consensusResult.Task == nil {
		return false, nil
	}
	beefyBlockNumber := uint64(consensusResult.Task.SignedCommitment.Commitment.BlockNumber)
	mmrAnchorRelayBlock := consensusResult.MMRAnchorRelayBlock
	beefyTarget := consensusResult.BeefyTarget

	rewardAddress, err := li.rewardAddressBytes()
	if err != nil {
		return false, fmt.Errorf("convert reward address: %w", err)
	}
	relayGasPriceWei, err := li.ethereumWriter.SuggestRelayGasPrice(ctx)
	if err != nil {
		return false, fmt.Errorf("suggest relay gas price: %w", err)
	}

	var (
		batch            *MessageBatchBuilder
		v2Count, v1Count int
		fiatShamirResult *beefy.ConsensusTaskResult
	)
	eg, egCtx := errgroup.WithContext(ctx)
	eg.Go(func() error {
		built, v2, v1, buildErr := li.buildFastPathMessageBatch(
			egCtx, mmrAnchorRelayBlock, rewardAddress, relayGasPriceWei, v1FeePiggyback, false,
		)
		if buildErr != nil {
			return buildErr
		}
		batch, v2Count, v1Count = built, v2, v1
		return nil
	})
	eg.Go(func() error {
		result, buildErr := li.beefyOnDemandRelay.BuildFiatShamirCalldataForTask(egCtx, consensusResult)
		if buildErr != nil {
			return fmt.Errorf("build beefy consensus calldata: %w", buildErr)
		}
		fiatShamirResult = result
		return nil
	})
	if err := eg.Wait(); err != nil {
		return false, err
	}

	// Same as instant: V2 pays the gate; V1-only waits for direct-relay / handover.
	if batch == nil || v2Count == 0 {
		log.WithFields(logBatchComposition(v2Count, v1Count)).
			Debug("[catch-up] No V2 pending for missed-WS fiat-shamir (V1-only deferred to direct-relay/handover)")
		return false, nil
	}
	if fiatShamirResult == nil || len(fiatShamirResult.Calldata) == 0 {
		log.WithField("beefyBlockNumber", beefyBlockNumber).Debug("[catch-up] No beefy consensus calldata produced")
		return false, nil
	}

	txOpts := li.ethereumWriter.conn.MakeTxOpts(ctx)
	li.applyMulticallGasFloor(txOpts, batch.MinGasSum)
	fiatShamirCalls := append([]contracts.Multicall3Call3{{
		Target:       beefyTarget,
		AllowFailure: false,
		CallData:     fiatShamirResult.Calldata,
	}}, batch.Calls...)

	profitable, metrics, basisFeeWei, pricingMode, err := li.evaluateFastPathProfitability(ctx, batch, txOpts.GasLimit, fiatShamirCalls)
	if err != nil {
		return false, err
	}
	fields := log.Fields{
		"beefyBlockNumber":         beefyBlockNumber,
		"mmrAnchorRelayBlock":      mmrAnchorRelayBlock,
		"pricingMode":              pricingMode,
		"estimatedExecutionFeeWei": metrics.EstimatedFeeWei.String(),
		"profitabilityBasisFeeWei": basisFeeWei.String(),
		"actualCollectedFeeWei":    batch.TotalFeeWei.String(),
		"isFiatShamirProfitable":   profitable,
	}
	for k, v := range logBatchComposition(v2Count, v1Count) {
		fields[k] = v
	}
	log.WithFields(fields).Info("[catch-up] Evaluated missed-WS fiat-shamir profitability")

	if !profitable {
		return false, nil
	}

	tx, err := li.submitAggregate3FastPath(ctx, txOpts, fiatShamirCalls)
	if err != nil {
		return false, err
	}
	log.WithFields(log.Fields{
		"txHash":                tx.Hash().Hex(),
		"beefyBlockNumber":      beefyBlockNumber,
		"actualCollectedFeeWei": batch.TotalFeeWei.String(),
	}).Info("[catch-up] Submitted missed-WS fiat-shamir Multicall3")
	return true, nil
}

// kickAsyncHandover starts a single-flight two-phase handover that piggybacks
// pending V2+V1 on the final Multicall3. Instant/direct-relay can still submit
// while this waits on Randao (submit serializer only covers broadcasts).
func (li *BeefyInstantSyncer) kickAsyncHandover(ctx context.Context, consensusResult *beefy.ConsensusTaskResult) {
	if consensusResult == nil || consensusResult.Task == nil || !consensusResult.IsHandover {
		return
	}
	if !li.handoverInFlight.CompareAndSwap(false, true) {
		log.WithField("beefyBlockNumber", consensusResult.Task.SignedCommitment.Commitment.BlockNumber).
			Debug("[catch-up] Handover already in flight; skipping kick")
		return
	}

	handoverCtx := context.WithoutCancel(ctx)

	go func() {
		defer li.handoverInFlight.Store(false)
		if err := li.runAsyncHandover(handoverCtx, consensusResult); err != nil {
			if error_tracking.IsTransientError(err) {
				li.handleTransientEthError(handoverCtx, err, "Transient error in async handover")
				return
			}
			log.WithError(err).Error("[catch-up] Async handover failed")
		}
	}()
}

func (li *BeefyInstantSyncer) runAsyncHandover(ctx context.Context, consensusResult *beefy.ConsensusTaskResult) error {
	beefyBlockNumber := uint64(consensusResult.Task.SignedCommitment.Commitment.BlockNumber)
	mmrAnchorRelayBlock := consensusResult.MMRAnchorRelayBlock

	relayGasPriceWei, err := li.ethereumWriter.SuggestRelayGasPrice(ctx)
	if err != nil {
		return fmt.Errorf("suggest relay gas price: %w", err)
	}

	lightClientBeefyBlockNumber, _, err := li.beefyListener.fetchLatestBeefyBlock(ctx)
	if err != nil {
		return fmt.Errorf("fetch latest beefy block for handover skip check: %w", err)
	}
	// Lag gate uses tip−LC (same metric as beefy cheap-gas), not commitment−LC.
	// Commitment stays the submit target; tip lag is what can exceed the 8h window.
	relayBeefyTip, err := li.beefyOnDemandRelay.RelayBeefyTip(ctx)
	if err != nil {
		return fmt.Errorf("fetch relay beefy tip for handover skip check: %w", err)
	}
	beefyBlockLag := handoverTipLag(relayBeefyTip, lightClientBeefyBlockNumber)

	rewardAddress, err := li.rewardAddressBytes()
	if err != nil {
		return fmt.Errorf("convert reward address: %w", err)
	}

	tasks, err := li.beefyListener.scanner.Scan(ctx, mmrAnchorRelayBlock)
	if err != nil {
		return fmt.Errorf("scan for handover messages: %w", err)
	}
	batch := NewMessageBatchBuilder(li.ethereumWriter.GatewayAddress(), false)
	v2Count, v1Count, err := li.appendMessagesFromScanTasks(
		ctx, batch, tasks, mmrAnchorRelayBlock, rewardAddress, relayGasPriceWei, v1FeePiggyback,
	)
	if err != nil {
		return err
	}

	// HIGH_GAS skips any handover (consensus-only or +messages) while tip lag is
	// inside the skip window. Past RelaySkipMaxLagBlocks we force the sync.
	// Message delivery at high gas still goes through instant/missed-WS when V2
	// clears the profitability gate, or direct-relay once the LC is ahead.
	if shouldSkipHandoverOnHighGas(relayGasPriceWei, beefyBlockLag, HighGasPriceThresholdWei, RelaySkipMaxLagBlocks) {
		log.WithFields(log.Fields{
			"beefyBlockNumber":            beefyBlockNumber,
			"commitmentBlock":             beefyBlockNumber,
			"relayBeefyTip":               relayBeefyTip,
			"lightClientBeefyBlockNumber": lightClientBeefyBlockNumber,
			"beefyBlockLag":               beefyBlockLag,
			"relayGasPriceWei":            relayGasPriceWei.String(),
			"skipThresholdWei":            fmt.Sprintf("%d", HighGasPriceThresholdWei),
			"v2MessageCount":              v2Count,
			"v1MessageCount":              v1Count,
		}).Info("[catch-up] Skipping handover because gas price is above hard threshold")
		return nil
	}
	if relayGasPriceWei.Cmp(new(big.Int).SetUint64(HighGasPriceThresholdWei)) > 0 {
		log.WithFields(log.Fields{
			"commitmentBlock":             beefyBlockNumber,
			"relayBeefyTip":               relayBeefyTip,
			"lightClientBeefyBlockNumber": lightClientBeefyBlockNumber,
			"beefyBlockLag":               beefyBlockLag,
			"relayGasPriceWei":            relayGasPriceWei.String(),
			"skipMaxLagBlocks":            RelaySkipMaxLagBlocks,
			"v2MessageCount":              v2Count,
			"v1MessageCount":              v1Count,
		}).Info("[catch-up] Forcing handover; gas above hard threshold but tip lag past skip window")
	}

	if len(batch.Calls) == 0 {
		log.WithField("beefyBlockNumber", beefyBlockNumber).Info("[catch-up] Submitting consensus-only handover via two-phase")
		return li.beefyOnDemandRelay.SyncRequestInTwoPhase(ctx, consensusResult.Task)
	}

	log.WithFields(log.Fields{
		"beefyBlockNumber":    beefyBlockNumber,
		"mmrAnchorRelayBlock": mmrAnchorRelayBlock,
		"v2MessageCount":      v2Count,
		"v1MessageCount":      v1Count,
	}).Info("[catch-up] Starting async two-phase handover with message piggyback")

	beefyCalldata, err := li.beefyOnDemandRelay.ExecuteTwoPhaseAndPackSubmitFinal(ctx, consensusResult)
	if err != nil {
		return fmt.Errorf("execute two-phase and pack submitFinal: %w", err)
	}
	if len(beefyCalldata) == 0 {
		log.WithField("beefyBlockNumber", beefyBlockNumber).Info("[catch-up] Consensus update completed or skipped before batched submitFinal")
		return nil
	}

	txOpts := li.ethereumWriter.conn.MakeTxOpts(ctx)
	li.applyMulticallGasFloor(txOpts, batch.MinGasSum)
	selectedCalls := append([]contracts.Multicall3Call3{{
		Target:       consensusResult.BeefyTarget,
		AllowFailure: false,
		CallData:     beefyCalldata,
	}}, batch.Calls...)

	metrics, err := li.simulateAggregate3(ctx, li.ethereumWriter.readConn, selectedCalls, txOpts.GasLimit)
	if err != nil {
		return fmt.Errorf("pre-send two-phase aggregate3 simulation failed: %w", err)
	}
	applyEffectiveGasLimit(txOpts, metrics)

	tx, err := li.submitAggregate3FastPath(ctx, txOpts, selectedCalls)
	if err != nil {
		return err
	}
	log.WithFields(log.Fields{
		"txHash":                tx.Hash().Hex(),
		"beefyBlockNumber":      beefyBlockNumber,
		"v2MessageCount":        v2Count,
		"v1MessageCount":        v1Count,
		"actualCollectedFeeWei": batch.TotalFeeWei.String(),
	}).Info("[catch-up] Submitted handover submitFinal + messages Multicall3")
	return nil
}
