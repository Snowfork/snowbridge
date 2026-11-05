package executionV3

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/prices"
	"github.com/snowfork/snowbridge/relayer/relays/xcmv5"
)

// ethBlockTimeSecs converts PROFITABILITY_MAX_DELAY_SECS into an Ethereum block count.
const ethBlockTimeSecs = 12

// How long a V2 nonce that could not be sent stays out of candidate selection, so
// it does not refill the force_batch cap on every scan. A soft-skipped dry-run
// retries sooner because it may pass once an earlier message lands.
const (
	underpaidRetry = 30 * time.Minute
	dryRunRetry    = 5 * time.Minute
)

// ErrAssetHubDryRunRejected marks a permanent AH dry_run_xcm reject (dead-letter).
// See skipAssetHubDryRunFailure for when it is returned vs soft-skipped.
var ErrAssetHubDryRunRejected = errors.New("asset hub dry-run rejected")

// v2PastMaxDelay is true when the message is old enough that fee/batch gates
// may be bypassed. MaxDelayBlocks(0) disables the valve.
func v2PastMaxDelay(ev *contracts.GatewayOutboundMessageAccepted, currentBlock uint64) bool {
	maxDelay := prices.MaxDelayBlocks(ethBlockTimeSecs)
	if maxDelay == 0 || ev == nil || currentBlock < ev.Raw.BlockNumber {
		return false
	}
	return currentBlock-ev.Raw.BlockNumber >= maxDelay
}

// skipAssetHubDryRunFailure never submits on AH dry_run_xcm reject. Soft-skips
// (ok=false, err=nil) inside max-delay so a send waiting on an earlier pending
// message can retry; returns ErrAssetHubDryRunRejected past max-delay, or
// immediately when max-delay is unset.
func skipAssetHubDryRunFailure(nonce uint64, ev *contracts.GatewayOutboundMessageAccepted, currentBlock uint64, ahErr string) (ok bool, err error) {
	maxDelay := prices.MaxDelayBlocks(ethBlockTimeSecs)
	pastDelay := v2PastMaxDelay(ev, currentBlock)
	if maxDelay > 0 && !pastDelay {
		log.WithFields(log.Fields{
			"nonce":          nonce,
			"error":          ahErr,
			"blockNumber":    evBlockNumber(ev),
			"currentBlock":   currentBlock,
			"maxDelayBlocks": maxDelay,
		}).Info("[instant-eth] V2 Asset Hub dry-run failed; soft-skipping until max-delay")
		return false, nil
	}
	log.WithFields(log.Fields{
		"nonce":        nonce,
		"error":        ahErr,
		"pastMaxDelay": pastDelay,
	}).Info("[instant-eth] V2 Asset Hub dry-run failed; dead-lettering")
	return false, fmt.Errorf("%w: %s", ErrAssetHubDryRunRejected, ahErr)
}

func evBlockNumber(ev *contracts.GatewayOutboundMessageAccepted) uint64 {
	if ev == nil {
		return 0
	}
	return ev.Raw.BlockNumber
}

// serviceFeeFromPayload SCALE-decodes the UI/service fee from Payload.Xcm (V5
// Definite DepositAsset of bridged ether to a configured recipient). Does not
// require the gas-estimator binary.
func (r *Relay) serviceFeeFromPayload(ev *contracts.GatewayOutboundMessageAccepted) *big.Int {
	if ev == nil || len(r.serviceFeeRecipients) == 0 {
		return new(big.Int)
	}
	return xcmv5.ExtractServiceFee(ev.Payload.Xcm.Data, r.serviceFeeRecipients)
}

// staticRewardWei is RelayerFee + decoded service fee (both in wei / ether base units).
func (r *Relay) staticRewardWei(ev *contracts.GatewayOutboundMessageAccepted) *big.Int {
	total := new(big.Int)
	if ev != nil && ev.Payload.RelayerFee != nil {
		total.Set(ev.Payload.RelayerFee)
	}
	total.Add(total, r.serviceFeeFromPayload(ev))
	return total
}

// passesStaticFeeFloor reports whether RelayerFee+serviceFee meets MinMessageRelayerFeeWei.
// When the floor is unset/zero, always passes. When gas estimation is enabled the
// floor is skipped here: add_tip is only in the estimator's V2RewardWei, so a
// RelayerFee-only check would drop tipped messages until max-delay.
func (r *Relay) passesStaticFeeFloor(ev *contracts.GatewayOutboundMessageAccepted) bool {
	if r.minMessageRelayerFee.Sign() <= 0 {
		return true
	}
	if r.gasEstimator.Enabled() {
		return true
	}
	return r.staticRewardWei(ev).Cmp(r.minMessageRelayerFee) >= 0
}

// checkV2Profitable keeps a V2 message that can be priced.
// Estimation off: passesStaticFeeFloor (or past max-delay). Estimation on:
// per-message estimate for AH dry-run / delivery / tip+fee; BH cost from
// selectProfitableBatch. inboundMsg must already carry ExecutionProof.
func (r *Relay) checkV2Profitable(
	ctx context.Context,
	ev *contracts.GatewayOutboundMessageAccepted,
	inboundMsg *parachain.MessageV3,
	source string,
	currentBlock uint64,
) (beneficiaries []string, econ *v2Economics, ok bool, err error) {
	pastDelay := v2PastMaxDelay(ev, currentBlock)
	if !r.gasEstimator.Enabled() {
		if !r.passesStaticFeeFloor(ev) {
			if !pastDelay {
				log.WithFields(log.Fields{
					"nonce":      ev.Nonce,
					"rewardWei":  r.staticRewardWei(ev).String(),
					"minWei":     r.minMessageRelayerFee.String(),
					"serviceFee": r.serviceFeeFromPayload(ev).String(),
				}).Info("[instant-eth] V2 message below static RelayerFee+serviceFee floor; skipping")
				return nil, nil, false, nil
			}
			log.WithFields(log.Fields{
				"nonce":        ev.Nonce,
				"rewardWei":    r.staticRewardWei(ev).String(),
				"minWei":       r.minMessageRelayerFee.String(),
				"blockNumber":  ev.Raw.BlockNumber,
				"currentBlock": currentBlock,
			}).Info("[instant-eth] V2 below static floor but past max delay; keeping")
		}
		return nil, &v2Economics{nonce: ev.Nonce, pastMaxDelay: pastDelay}, true, nil
	}

	relayerPublicKey := r.keypair.PublicKey()
	estimate, err := r.gasEstimator.EstimateGas(ctx, ev, inboundMsg, source, relayerPublicKey)
	if err != nil {
		return nil, nil, false, fmt.Errorf("gas estimation: %w", err)
	}

	if !estimate.AssetHubAccepted() {
		ok, err = skipAssetHubDryRunFailure(ev.Nonce, ev, currentBlock, gasEstimateErr(estimate.AssetHubDryRunError))
		return nil, nil, ok, err
	}

	delivery := new(big.Int).Set(&estimate.BridgeHub.DeliveryFeeInEther)
	var relayerFee *big.Int
	if ev != nil && ev.Payload.RelayerFee != nil {
		relayerFee = ev.Payload.RelayerFee
	}
	econ = &v2Economics{
		nonce:        ev.Nonce,
		reward:       estimate.V2RewardWei(relayerFee),
		delivery:     delivery,
		pastMaxDelay: pastDelay,
	}
	log.WithFields(log.Fields{
		"nonce":        ev.Nonce,
		"rewardWei":    econ.reward.String(),
		"deliveryWei":  econ.delivery.String(),
		"pastMaxDelay": pastDelay,
	}).Info("[instant-eth] V2 message priced for batch gate")
	return estimate.Beneficiaries, econ, true, nil
}

func gasEstimateErr(msg *string) string {
	if msg == nil {
		return ""
	}
	return *msg
}

// coversOwnDelivery is false when the reward does not pay the message's own Asset
// Hub delivery fee. Such a message only lowers the batch total, so it is left out
// until max-delay.
func (e *v2Economics) coversOwnDelivery() bool {
	if e == nil || e.reward == nil || e.delivery == nil || e.pastMaxDelay {
		return true
	}
	return e.reward.Cmp(e.delivery) >= 0
}

func (r *Relay) deferNonce(nonce uint64, retryAt time.Time) {
	r.deferredMu.Lock()
	defer r.deferredMu.Unlock()
	r.deferredV2[nonce] = retryAt
}

func (r *Relay) isDeferred(nonce uint64, now time.Time) bool {
	r.deferredMu.Lock()
	defer r.deferredMu.Unlock()
	retryAt, ok := r.deferredV2[nonce]
	if !ok {
		return false
	}
	if !now.Before(retryAt) {
		delete(r.deferredV2, nonce)
		return false
	}
	return true
}

// pruneDeferred forgets nonces that are no longer pending.
func (r *Relay) pruneDeferred(pending []uint64) {
	keep := make(map[uint64]struct{}, len(pending))
	for _, n := range pending {
		keep[n] = struct{}{}
	}
	r.deferredMu.Lock()
	defer r.deferredMu.Unlock()
	for n := range r.deferredV2 {
		if _, ok := keep[n]; !ok {
			delete(r.deferredV2, n)
		}
	}
}
