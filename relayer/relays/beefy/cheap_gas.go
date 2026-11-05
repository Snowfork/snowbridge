package beefy

import (
	"context"
	"fmt"
	"math/big"
	"os"
	"strconv"
	"time"

	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

const (
	relayBlockTimeSecs = 6
)

var (
	// BeefyFiatShamirMaxWei: gas below this → submitFiatShamir (default 0.1 gwei).
	BeefyFiatShamirMaxWei = getEnvUint64("BEEFY_FIAT_SHAMIR_MAX_WEI", 100_000_000)
	// BeefyWaitMinWei: gas at/above this → park (unless lag past skip window). Default 0.25 gwei
	// matches parachain-v3 HIGH_GAS_PRICE_THRESHOLD_WEI.
	BeefyWaitMinWei = getEnvUint64("BEEFY_WAIT_MIN_WEI", 250_000_000)
	// BeefySkipMaxLagSecs: how far the LC may lag before high-gas park is abandoned
	// and two-phase is forced. Same default as parachain-v3 RELAY_SKIP_MAX_LAG_SECS.
	BeefySkipMaxLagSecs   = getEnvUint64("BEEFY_SKIP_MAX_LAG_SECS", 28800) // 8h
	BeefySkipMaxLagBlocks = BeefySkipMaxLagSecs / relayBlockTimeSecs
	// BeefyPollIntervalSecs between loop iterations when idle / parked.
	// Default 15m — session handovers are ~4h apart; aggressive polling mostly burns RPC.
	BeefyPollIntervalSecs = getEnvUint64("BEEFY_POLL_INTERVAL_SECS", 900)
)

type beefyGasPath int

const (
	pathFiatShamir beefyGasPath = iota
	pathTwoPhase
	pathWait
	pathForceTwoPhase
)

func (p beefyGasPath) String() string {
	switch p {
	case pathFiatShamir:
		return "fiat-shamir"
	case pathTwoPhase:
		return "two-phase"
	case pathWait:
		return "wait"
	case pathForceTwoPhase:
		return "force-two-phase"
	default:
		return "unknown"
	}
}

func selectBeefyGasPath(gasWei *big.Int, lagBlocks uint64) beefyGasPath {
	if gasWei == nil {
		gasWei = big.NewInt(0)
	}
	fiatMax := new(big.Int).SetUint64(BeefyFiatShamirMaxWei)
	waitMin := new(big.Int).SetUint64(BeefyWaitMinWei)

	if gasWei.Cmp(fiatMax) < 0 {
		return pathFiatShamir
	}
	if gasWei.Cmp(waitMin) < 0 {
		return pathTwoPhase
	}
	// g >= wait threshold
	if lagBlocks <= BeefySkipMaxLagBlocks {
		return pathWait
	}
	return pathForceTwoPhase
}

// StartCheapGasLoop connects and continuously submits validator-set handover
// BEEFY updates only (never routine tip-only updates, never Gateway messages),
// using the three-tier gas policy (fiat-shamir / two-phase / park-with-lag-cap).
func (relay *OnDemandRelay) StartCheapGasLoop(ctx context.Context, eg *errgroup.Group) error {
	if err := relay.InitializeOnDemandSync(ctx, eg); err != nil {
		return err
	}

	interval := time.Second * time.Duration(BeefyPollIntervalSecs)
	if interval < time.Second {
		interval = time.Second
	}
	ticker := time.NewTicker(interval)

	eg.Go(func() error {
		defer ticker.Stop()
		submitEndpoint := ""
		if cfg := relay.GetConfig(); cfg != nil {
			submitEndpoint = cfg.Sink.Ethereum.SubmitEndpoint
		}
		log.WithFields(log.Fields{
			"fiatShamirMaxWei": BeefyFiatShamirMaxWei,
			"waitMinWei":       BeefyWaitMinWei,
			"skipMaxLagSecs":   BeefySkipMaxLagSecs,
			"skipMaxLagBlocks": BeefySkipMaxLagBlocks,
			"pollIntervalSecs": BeefyPollIntervalSecs,
			"beefyClient":      relay.BeefyClientAddress().Hex(),
			"mode":             "handover-only",
			"submitEndpoint":   submitEndpoint,
		}).Info("beefy cheap-gas loop started")

		for {
			if err := relay.cheapGasTick(ctx); err != nil {
				log.WithError(err).Warn("beefy cheap-gas tick failed")
			}
			select {
			case <-ctx.Done():
				return nil
			case <-ticker.C:
			}
		}
	})
	return nil
}

// cheapGasTick finds a pending validator-set handover first; only then applies
// the gas policy and (optionally) submits. Non-handover commitments never reach
// the Ethereum writer.
func (relay *OnDemandRelay) cheapGasTick(ctx context.Context) error {
	state, tip, task, lag, err := relay.findPendingHandover(ctx)
	if err != nil {
		return err
	}
	if task == nil {
		return nil
	}

	gasWei, err := relay.ethereumWriter.SuggestBaseFee(ctx)
	if err != nil {
		return fmt.Errorf("read eth base fee: %w", err)
	}

	path := selectBeefyGasPath(gasWei, lag)
	leafNext := uint64(task.Proof.Leaf.BeefyNextAuthoritySet.ID)
	logger := log.WithFields(log.Fields{
		"event":                  "beefy_cheap_gas",
		"path":                   path.String(),
		"gasWei":                 gasWei.String(),
		"latestBeefyBlock":       state.LatestBeefyBlock,
		"relayBeefyTip":          tip,
		"lagBlocks":              lag,
		"commitmentBlock":        uint64(task.SignedCommitment.Commitment.BlockNumber),
		"commitmentValidatorSet": uint64(task.SignedCommitment.Commitment.ValidatorSetID),
		"leafNextValidatorSetID": leafNext,
		"lightClientNextSetID":   state.NextValidatorSetID,
	})

	switch path {
	case pathWait:
		logger.Info("parking handover; gas above wait threshold and lag within skip window")
		return nil
	case pathFiatShamir:
		logger.Info("submitting fiat-shamir validator-set handover")
		return relay.submitHandoverOnly(ctx, state, task, false)
	case pathTwoPhase, pathForceTwoPhase:
		if path == pathForceTwoPhase {
			logger.Info("forcing two-phase validator-set handover; gas high but lag past skip window")
		} else {
			logger.Info("submitting two-phase validator-set handover")
		}
		return relay.submitHandoverOnly(ctx, state, task, true)
	default:
		return fmt.Errorf("unknown beefy gas path %v", path)
	}
}

// isHandoverRequest reports whether submitting task would rotate the light
// client's next validator set (same predicate as ConsensusTaskResult.IsHandover).
func isHandoverRequest(state *BeefyClientState, task *Request) bool {
	if state == nil || task == nil {
		return false
	}
	return uint64(task.Proof.Leaf.BeefyNextAuthoritySet.ID) > state.NextValidatorSetID
}

// findPendingHandover returns a proveable forward commitment only when it is a
// validator-set handover. Otherwise logs skip/idle and returns a nil task.
func (relay *OnDemandRelay) findPendingHandover(ctx context.Context) (*BeefyClientState, uint64, *Request, uint64, error) {
	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return nil, 0, nil, 0, fmt.Errorf("query beefy client state: %w", err)
	}
	tip, err := relay.RelayBeefyTip(ctx)
	if err != nil {
		return nil, 0, nil, 0, fmt.Errorf("fetch beefy tip: %w", err)
	}
	var lag uint64
	if tip > state.LatestBeefyBlock {
		lag = tip - state.LatestBeefyBlock
	}
	if lag == 0 {
		log.WithFields(log.Fields{
			"event":            "beefy_cheap_gas",
			"latestBeefyBlock": state.LatestBeefyBlock,
			"relayBeefyTip":    tip,
		}).Debug("light client at tip; idle")
		return state, tip, nil, 0, nil
	}

	task, err := relay.polkadotListener.generateBeefyUpdateForwardFromLightClient(state, tip)
	if err != nil {
		return nil, 0, nil, 0, err
	}
	if task == nil {
		log.Info("Light client already at or past relay BEEFY tip")
		return state, tip, nil, lag, nil
	}

	leafNext := uint64(task.Proof.Leaf.BeefyNextAuthoritySet.ID)
	commitmentSet := uint64(task.SignedCommitment.Commitment.ValidatorSetID)
	commitmentBlock := uint64(task.SignedCommitment.Commitment.BlockNumber)
	if !isHandoverRequest(state, task) {
		log.WithFields(log.Fields{
			"event":                  "beefy_skip_non_handover",
			"commitmentBlock":        commitmentBlock,
			"commitmentValidatorSet": commitmentSet,
			"leafNextValidatorSetID": leafNext,
			"lightClientNextSetID":   state.NextValidatorSetID,
			"lagBlocks":              lag,
		}).Info("no validator-set handover in window; skipping (handover-only service)")
		return state, tip, nil, lag, nil
	}

	log.WithFields(log.Fields{
		"event":                  "beefy_handover_pending",
		"commitmentBlock":        commitmentBlock,
		"commitmentValidatorSet": commitmentSet,
		"leafNextValidatorSetID": leafNext,
		"lightClientNextSetID":   state.NextValidatorSetID,
		"lagBlocks":              lag,
	}).Info("validator-set handover pending")
	return state, tip, task, lag, nil
}

// submitHandoverOnly re-checks the handover predicate then submits. Non-handover
// tasks are refused (defense in depth for any future caller).
func (relay *OnDemandRelay) submitHandoverOnly(ctx context.Context, state *BeefyClientState, task *Request, twoPhase bool) error {
	if !isHandoverRequest(state, task) {
		return fmt.Errorf("refusing non-handover beefy submit (leafNext=%d lcNext=%d)",
			uint64(task.Proof.Leaf.BeefyNextAuthoritySet.ID), state.NextValidatorSetID)
	}
	if twoPhase {
		return relay.SyncRequestInTwoPhase(ctx, task)
	}
	return relay.syncBeefyUpdateFiatShamir(ctx, task)
}

// SyncNextHandoverFromLightClient finds the farthest forward-compatible BEEFY
// commitment after the on-chain tip and submits it only when it is a validator-set
// handover. Non-handover commitments (same NextAuthoritySetID) are skipped.
// twoPhase selects submitInitial+commit+submitFinal vs submitFiatShamir.
func (relay *OnDemandRelay) SyncNextHandoverFromLightClient(ctx context.Context, twoPhase bool) error {
	state, _, task, _, err := relay.findPendingHandover(ctx)
	if err != nil {
		return err
	}
	if task == nil {
		return nil
	}
	mode := "fiat-shamir"
	if twoPhase {
		mode = "two-phase"
	}
	log.WithFields(log.Fields{
		"event": "beefy_handover",
		"mode":  mode,
	}).Info("submitting validator-set handover")
	return relay.submitHandoverOnly(ctx, state, task, twoPhase)
}

// SyncNextFromLightClientTwoPhase is SyncNextFromLightClient but uses the
// two-phase submit path instead of fiat-shamir. Handover-only.
func (relay *OnDemandRelay) SyncNextFromLightClientTwoPhase(ctx context.Context) error {
	return relay.SyncNextHandoverFromLightClient(ctx, true)
}

// SuggestBaseFee returns the latest block base fee (preferred gas signal for
// EIP-1559 mainnet). Falls back to SuggestGasPrice if base fee is unavailable.
func (wr *EthereumWriter) SuggestBaseFee(ctx context.Context) (*big.Int, error) {
	header, err := wr.conn.Client().HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, err
	}
	if header != nil && header.BaseFee != nil {
		return new(big.Int).Set(header.BaseFee), nil
	}
	return wr.conn.Client().SuggestGasPrice(ctx)
}

func getEnvUint64(key string, defaultVal uint64) uint64 {
	raw := os.Getenv(key)
	if raw == "" {
		return defaultVal
	}
	v, err := strconv.ParseUint(raw, 10, 64)
	if err != nil {
		log.WithError(err).WithField("key", key).Warn("invalid uint64 env; using default")
		return defaultVal
	}
	return v
}
