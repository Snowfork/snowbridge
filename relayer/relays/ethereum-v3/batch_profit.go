package executionV3

import (
	"context"
	"fmt"
	"math/big"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/gasestimator"
)

const maxBatchDryRunPasses = 4

// batchCall is one inner call of the force_batch ethereum-v3 submits.
// v2 is set only for EthereumInboundQueueV2.submit.
type batchCall struct {
	name    string
	payload interface{}
	v2      *v2Economics
}

// v2Economics is the per-message half of `estimate message`: reward and the
// Asset Hub delivery fee. pastMaxDelay means PROFITABILITY_MAX_DELAY_SECS has
// elapsed since the Ethereum event block, so the batch fee gate may keep this
// call even when the batch is unprofitable.
type v2Economics struct {
	nonce        uint64
	reward       *big.Int
	delivery     *big.Int
	pastMaxDelay bool
}

// selectProfitableBatch dry-runs the assembled force_batch (the real BH
// extrinsic ethereum-v3 submits, optionally including a beacon update), drops
// inner calls that fail, and keeps V2 calls only when their combined reward
// exceeds that one batch extrinsic fee plus their combined AH delivery fees.
// V2 calls past PROFITABILITY_MAX_DELAY_SECS are kept even when the batch is
// unprofitable. This is the sole Bridge Hub cost/dry-run source of truth;
// estimate message does not quote a bare-submit fee.
//
// When gas estimation is disabled the static per-message floor already ran,
// and this returns calls unchanged.
func (r *Relay) selectProfitableBatch(ctx context.Context, calls []batchCall) ([]batchCall, error) {
	if !r.gasEstimator.Enabled() || len(calls) == 0 {
		return calls, nil
	}

	priced := make([]batchCall, 0, len(calls))
	for _, c := range calls {
		if c.name == "EthereumInboundQueueV2.submit" && c.v2 == nil {
			log.Warn("[instant-eth] dropping V2 call with no gas quote")
			continue
		}
		priced = append(priced, c)
	}
	calls = priced

	var fee *big.Int
	for pass := 0; pass < maxBatchDryRunPasses; pass++ {
		if len(calls) == 0 {
			return nil, nil
		}
		dry, err := r.dryRunBatch(ctx, calls)
		if err != nil {
			return nil, err
		}
		if len(dry.Items) == 0 && !dry.DryRunSuccess {
			return nil, fmt.Errorf("force_batch dry-run failed: %s", gasEstimateErr(dry.DryRunError))
		}
		failed := failedBatchIndexes(len(calls), dry.Items)
		if len(failed) == 0 {
			if !dry.DryRunSuccess {
				return nil, fmt.Errorf("force_batch dry-run failed: %s", gasEstimateErr(dry.DryRunError))
			}
			fee = new(big.Int).Set(&dry.ExtrinsicFeeInEther)
			break
		}
		for _, i := range failed {
			fields := log.Fields{
				"call":          calls[i].name,
				"index":         i,
				"dryRunError":   gasEstimateErr(dry.DryRunError),
				"dryRunSuccess": dry.DryRunSuccess,
			}
			if calls[i].v2 != nil {
				fields["nonce"] = calls[i].v2.nonce
			}
			if i < len(dry.Items) {
				fields["itemSuccess"] = dry.Items[i].Success
				fields["itemError"] = gasEstimateErr(dry.Items[i].Error)
			}
			log.WithFields(fields).Info("[instant-eth] dropping inner call that failed the batch dry-run")
		}
		calls = withoutIndexes(calls, failed)
		fee = nil
	}
	if fee == nil {
		return nil, fmt.Errorf("force_batch dry-run still failing after dropping inner calls")
	}

	reward, delivery, n := v2BatchTotals(calls)
	if n > 0 && !v2BatchCoversCost(calls, fee) {
		var kept []batchCall
		forced := 0
		for _, c := range calls {
			if c.v2 == nil {
				kept = append(kept, c)
				continue
			}
			if c.v2.pastMaxDelay {
				kept = append(kept, c)
				forced++
			}
		}
		fields := log.Fields{
			"rewardWei":   reward.String(),
			"deliveryWei": delivery.String(),
			"batchFeeWei": fee.String(),
			"v2Count":     n,
			"forcedV2":    forced,
		}
		if forced > 0 {
			log.WithFields(fields).Info("[instant-eth] V2 batch unprofitable; keeping messages past max delay")
		} else {
			log.WithFields(fields).Info("[instant-eth] V2 rewards do not cover the batch fee plus delivery fees; dropping V2 calls")
		}
		return kept, nil
	}
	if n > 0 {
		log.WithFields(log.Fields{
			"rewardWei":   reward.String(),
			"deliveryWei": delivery.String(),
			"batchFeeWei": fee.String(),
			"v2Count":     n,
		}).Info("[instant-eth] batch reward covers fee plus delivery fees")
	}
	return calls, nil
}

func (r *Relay) dryRunBatch(ctx context.Context, calls []batchCall) (*gasestimator.BatchDryRun, error) {
	hexCalls := make([]string, len(calls))
	for i, c := range calls {
		encoded, err := r.writer.EncodeRuntimeCall(c.name, c.payload)
		if err != nil {
			return nil, fmt.Errorf("encode %s: %w", c.name, err)
		}
		hexCalls[i] = fmt.Sprintf("0x%x", encoded)
	}
	return r.gasEstimator.EstimateBatch(ctx, hexCalls, r.keypair.PublicKey())
}

func v2BatchTotals(calls []batchCall) (reward, delivery *big.Int, n int) {
	reward = new(big.Int)
	delivery = new(big.Int)
	for _, c := range calls {
		if c.v2 == nil {
			continue
		}
		n++
		if c.v2.reward != nil {
			reward.Add(reward, c.v2.reward)
		}
		if c.v2.delivery != nil {
			delivery.Add(delivery, c.v2.delivery)
		}
	}
	return reward, delivery, n
}

// v2BatchCoversCost is true when there is no V2 call, or when the sum of V2
// rewards is strictly greater than one batch extrinsic fee plus the sum of
// those messages' Asset Hub delivery fees. Beacon and V1 contribute fee and
// no reward, so they are inside batchFee exactly once.
func v2BatchCoversCost(calls []batchCall, batchFee *big.Int) bool {
	reward, delivery, n := v2BatchTotals(calls)
	if n == 0 {
		return true
	}
	cost := new(big.Int)
	if batchFee != nil {
		cost.Set(batchFee)
	}
	cost.Add(cost, delivery)
	return reward.Cmp(cost) > 0
}

func failedBatchIndexes(callCount int, items []gasestimator.BatchItemDryRun) []int {
	ok := make([]bool, callCount)
	seen := make([]bool, callCount)
	for _, it := range items {
		i := int(it.Index)
		if i < 0 || i >= callCount {
			continue
		}
		seen[i] = true
		ok[i] = it.Success
	}
	var failed []int
	for i := 0; i < callCount; i++ {
		if !seen[i] || !ok[i] {
			failed = append(failed, i)
		}
	}
	return failed
}

func withoutIndexes(calls []batchCall, drop []int) []batchCall {
	if len(drop) == 0 {
		return calls
	}
	skip := make(map[int]struct{}, len(drop))
	for _, i := range drop {
		skip[i] = struct{}{}
	}
	out := make([]batchCall, 0, len(calls))
	for i, c := range calls {
		if _, ok := skip[i]; ok {
			continue
		}
		out = append(out, c)
	}
	return out
}

func callsToForceBatch(calls []batchCall) ([]string, []interface{}) {
	names := make([]string, len(calls))
	payloads := make([]interface{}, len(calls))
	for i, c := range calls {
		names[i] = c.name
		payloads[i] = c.payload
	}
	return names, payloads
}

func batchHasMessage(calls []batchCall) bool {
	for _, c := range calls {
		if c.name != "EthereumBeaconClient.submit" {
			return true
		}
	}
	return false
}
