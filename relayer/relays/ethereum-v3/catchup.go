package executionV3

import (
	"context"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
)

// submitBeaconCatchUp is the Header FinalizedSubmitter: attach any V1 (always) and
// V2 messages that prove against this update or existing on-chain finality.
// With gas estimation on, the assembled force_batch is dry-run and V2 calls stay
// only when their combined reward covers one batch fee plus their delivery fees.
// Beacon+messages go in ForceBatch; solo LC uses a bare EthereumBeaconClient.submit
// so FreeHeadersInterval / sync-committee Pays::No can apply (force_batch always pays).
//
// ParachainWriter already serializes WriteToParachainAndWatch / ForceBatch via mutex,
// so tip and catch-up cannot double-sign.
func (r *Relay) submitBeaconCatchUp(ctx context.Context, update scale.Update) error {
	// Same ethMu.RLock as tip scans: reconnect cannot rebind mid catch-up submit.
	r.ethMu.RLock()
	defer r.ethMu.RUnlock()

	onchain, err := r.writer.GetLastFinalizedHeaderStateAtBestBlock()
	if err != nil {
		// Message proofs need this slot. The LC update does not, and a non-transient
		// error here would exit Header.Sync and stop the relay.
		log.WithError(err).Warn("[instant-eth] catch-up: on-chain state unavailable; submitting LC alone")
		return r.submitLightClientOnly(ctx, update)
	}

	// Execution tip for log scans: prefer the update's committed execution block when
	// we can resolve it; otherwise use the latest finalized beacon checkpoint's EL tip.
	executionBlockNumber, err := r.executionBlockNumberForCatchUp(ctx, update)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] catch-up: could not resolve EL tip; submitting LC alone")
		return r.submitLightClientOnly(ctx, update)
	}

	v1Cands, err := r.collectV1Piggybacks(ctx, executionBlockNumber, onchain.BeaconSlot, &update)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] catch-up: V1 collect failed; submitting LC alone")
		return r.submitLightClientOnly(ctx, update)
	}

	v2Cands, err := r.collectV2ForCatchUp(ctx, executionBlockNumber, onchain.BeaconSlot, &update)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] catch-up: V2 collect failed; submitting LC alone")
		return r.submitLightClientOnly(ctx, update)
	}

	calls := []batchCall{{
		name:    "EthereumBeaconClient.submit",
		payload: update.Payload,
	}}

	for _, c := range v2Cands {
		c.inboundMsg.Proof.ExecutionProof = c.proof.HeaderPayload
		submitPayload, err := r.paraconn.GenerateSubmitMessagePayload(c.inboundMsg)
		if err != nil {
			log.WithError(err).WithField("nonce", c.ev.Nonce).Warn("[instant-eth] catch-up: V2 payload failed; submitting LC alone")
			return r.submitLightClientOnly(ctx, update)
		}
		calls = append(calls, batchCall{
			name:    "EthereumInboundQueueV2.submit",
			payload: submitPayload,
			v2:      c.quote,
		})
		log.WithField("nonce", c.ev.Nonce).Info("[instant-eth] catch-up: Including EthereumInboundQueueV2.submit")
	}
	v1Extrinsics, v1Payloads, err := r.appendV1ToBatch(nil, nil, v1Cands)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] catch-up: V1 payload failed; submitting LC alone")
		return r.submitLightClientOnly(ctx, update)
	}
	for i := range v1Extrinsics {
		calls = append(calls, batchCall{name: v1Extrinsics[i], payload: v1Payloads[i]})
	}

	calls, err = r.selectProfitableBatch(ctx, calls)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] catch-up: batch dry-run failed; submitting LC alone")
		return r.submitLightClientOnly(ctx, update)
	}
	if len(calls) == 0 {
		log.Warn("[instant-eth] catch-up: batch dry-run rejected the beacon update; not submitting")
		return nil
	}

	// Solo LC: bare submit preserves pallet Pays::No refunds. force_batch does not.
	if !batchHasMessage(calls) {
		return r.submitLightClientOnly(ctx, update)
	}

	extrinsics, payloads := callsToForceBatch(calls)
	var v1Left, v2Left int
	for _, c := range calls {
		switch c.name {
		case "EthereumInboundQueue.submit":
			v1Left++
		case "EthereumInboundQueueV2.submit":
			v2Left++
		}
	}
	log.WithFields(log.Fields{
		"slot":  update.Payload.FinalizedHeader.Slot,
		"v1":    v1Left,
		"v2":    v2Left,
		"calls": len(extrinsics),
	}).Info("[instant-eth] catch-up ForceBatch (LC + messages)")

	return r.writer.ForceBatch(ctx, extrinsics, payloads)
}

func (r *Relay) submitLightClientOnly(ctx context.Context, update scale.Update) error {
	log.WithField("slot", update.Payload.FinalizedHeader.Slot).
		Info("[instant-eth] catch-up bare EthereumBeaconClient.submit")
	return r.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.submit", update.Payload)
}

func (r *Relay) executionBlockNumberForCatchUp(ctx context.Context, update scale.Update) (uint64, error) {
	root := update.FinalizedHeaderBlockRoot
	beaconBlock, err := r.beaconAPI.GetBeaconBlock(root)
	if err != nil {
		return 0, err
	}
	return r.executionBlockNumberFromBeaconBlock(ctx, &beaconBlock, root)
}

// collectV2ForCatchUp gathers unprocessed V2 messages passing the static fee gate
// that prove against on-chain LC or the catch-up update being submitted.
func (r *Relay) collectV2ForCatchUp(
	ctx context.Context,
	executionBlockNumber uint64,
	onchainFinalizedSlot uint64,
	update *scale.Update,
) ([]batchCandidate, error) {
	ethNonce, err := r.fetchEthereumNonceAtBlock(ctx, executionBlockNumber)
	if err != nil {
		return nil, err
	}
	paraNonces, err := r.fetchUnprocessedParachainNonces(ethNonce)
	if err != nil {
		return nil, err
	}
	paraNonces = r.filterSkippedNonces(paraNonces)
	if len(paraNonces) == 0 {
		return nil, nil
	}

	nonceSet := make(map[uint64]struct{}, len(paraNonces))
	for _, n := range paraNonces {
		nonceSet[n] = struct{}{}
	}

	events, err := r.findEvents(ctx, executionBlockNumber, paraNonces)
	if err != nil {
		return nil, err
	}

	maxMessages := int(r.writer.MaxWatchedExtrinsics()) - 1 - 1 // leave room for LC + V1
	if maxMessages <= 0 {
		maxMessages = 1
	}

	var out []batchCandidate
	cp := cacheProofFromUpdate(update)
	now := time.Now()

	for _, ev := range events {
		if len(out) >= maxMessages {
			break
		}
		if _, ok := nonceSet[ev.Nonce]; !ok {
			continue
		}
		if !r.passesStaticFeeFloor(ev) && !v2PastMaxDelay(ev, executionBlockNumber) {
			continue
		}
		if r.isDeferred(ev.Nonce, now) && !v2PastMaxDelay(ev, executionBlockNumber) {
			continue
		}

		c, err := r.buildCandidate(ctx, ev, onchainFinalizedSlot)
		if err != nil {
			log.WithError(err).WithField("nonce", ev.Nonce).Debug("[instant-eth] catch-up V2 candidate skipped")
			continue
		}

		var headerUpdate scale.HeaderUpdatePayload
		if c.needsConsensus {
			if c.headerSlot > cp.Slot {
				continue
			}
			headerUpdate, err = r.beaconHeader.GetHeaderUpdate(c.parentBeaconRoot, &cp)
		} else {
			headerUpdate, err = r.beaconHeader.GetHeaderUpdateAgainstOnchainFinalized(c.parentBeaconRoot)
		}
		if err != nil {
			continue
		}
		c.proof = scale.ProofPayload{HeaderPayload: headerUpdate, FinalizedPayload: update}
		c.inboundMsg.Proof.ExecutionProof = headerUpdate

		beneficiaries, quote, profitable, err := r.checkV2Profitable(ctx, c.ev, c.inboundMsg, c.source, executionBlockNumber)
		if err != nil {
			// Permanent → dead-letter; transient → retry next catch-up.
			if handled := r.handleCandidateError(ev.Nonce, err); handled != nil {
				log.WithError(handled).WithField("nonce", ev.Nonce).Debug("[instant-eth] catch-up V2 profitability transient; skipping")
			}
			continue
		}
		if !profitable {
			r.deferNonce(ev.Nonce, now.Add(dryRunRetry))
			continue
		}
		if !quote.coversOwnDelivery() {
			r.deferNonce(ev.Nonce, now.Add(underpaidRetry))
			continue
		}
		c.quote = quote

		sourceBanned, err := r.ofac.IsBanned(c.source, beneficiaries)
		if err != nil {
			return nil, err
		}
		if sourceBanned {
			continue
		}
		out = append(out, *c)
	}
	return out, nil
}
