package executionV3

import (
	"context"
	"fmt"
	"math/big"
	"sort"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	goethtypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	contractsv1 "github.com/snowfork/snowbridge/relayer/contracts/v1"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/execution"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
)

// v1Candidate is a V1 inbound message ready to piggyback on a force_batch.
type v1Candidate struct {
	ev         *contractsv1.GatewayOutboundMessageAccepted
	inboundMsg *parachain.MessageV3
	proof      scale.HeaderUpdatePayload
	source     string
}

func (r *Relay) fetchV1ParachainNonce() (uint64, error) {
	encoded, err := types.EncodeToBytes(r.config.Source.ChannelID)
	if err != nil {
		return 0, err
	}
	key, err := types.CreateStorageKey(r.paraconn.Metadata(), "EthereumInboundQueue", "Nonce", encoded, nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key for EthereumInboundQueue.Nonce: %w", err)
	}
	var nonce types.U64
	ok, err := r.paraconn.API().RPC.State.GetStorageLatest(key, &nonce)
	if err != nil {
		return 0, fmt.Errorf("fetch EthereumInboundQueue.Nonce: %w", err)
	}
	if !ok {
		return 0, nil
	}
	return uint64(nonce), nil
}

func (r *Relay) fetchV1EthereumNonceAtBlock(ctx context.Context, blockNumber uint64) (uint64, error) {
	if r.gatewayContractV1 == nil {
		return 0, fmt.Errorf("V1 gateway not bound")
	}
	opts := &bind.CallOpts{Context: ctx, BlockNumber: new(big.Int).SetUint64(blockNumber)}
	_, ethOutboundNonce, err := r.gatewayContractV1.ChannelNoncesOf(opts, r.config.Source.ChannelID)
	if err != nil {
		return 0, fmt.Errorf("ChannelNoncesOf: %w", err)
	}
	return ethOutboundNonce, nil
}

// findV1Events returns the channel's V1 events for nonces, reading only the
// block that emitted each one (see findEvents).
func (r *Relay) findV1Events(
	ctx context.Context,
	latestFinalizedBlockNumber uint64,
	nonces []uint64,
) ([]*contractsv1.GatewayOutboundMessageAccepted, error) {
	if r.gatewayContractV1 == nil {
		return nil, fmt.Errorf("V1 gateway not bound")
	}
	blocks, err := r.resolveNonceBlocks(ctx, latestFinalizedBlockNumber, nonces, r.v1NonceBlocks, r.fetchV1EthereumNonceAtBlock)
	if err != nil {
		return nil, fmt.Errorf("resolve V1 nonce blocks: %w", err)
	}

	var allEvents []*contractsv1.GatewayOutboundMessageAccepted
	found := make(map[uint64]struct{}, len(blocks))
	for _, block := range distinctBlocks(blocks) {
		end := block
		opts := &bind.FilterOpts{Start: block, End: &end, Context: ctx}
		iter, err := r.gatewayContractV1.FilterOutboundMessageAccepted(opts, [][32]byte{r.config.Source.ChannelID}, [][32]byte{})
		if err != nil {
			return nil, fmt.Errorf("filter V1 events: %w", err)
		}
		for iter.Next() {
			if b, ok := blocks[iter.Event.Nonce]; ok && b == block {
				allEvents = append(allEvents, iter.Event)
				found[iter.Event.Nonce] = struct{}{}
			}
		}
		err = iter.Error()
		iter.Close()
		if err != nil {
			return nil, fmt.Errorf("filter V1 events: %w", err)
		}
	}
	// InboundQueue requires sequential nonces; keep ascending order before the batch cap.
	sort.SliceStable(allEvents, func(i, j int) bool {
		return allEvents[i].Nonce < allEvents[j].Nonce
	})
	r.forgetMissingNonces(r.v1NonceBlocks, blocks, found)
	return allEvents, nil
}

// collectV1Piggybacks gathers unprocessed V1 messages that already prove against
// on-chain LC (or against checkpoint when provided). V1 always piggybacks — no fee gate.
// Nonces are collected contiguously from paraNonce+1 (InboundQueue is sequential): the
// first not-ready or banned nonce stops the run so we never jump ahead.
func (r *Relay) collectV1Piggybacks(
	ctx context.Context,
	executionBlockNumber uint64,
	onchainFinalizedSlot uint64,
	checkpoint *scale.Update,
) ([]v1Candidate, error) {
	if !r.config.V1Enabled() {
		return nil, nil
	}

	paraNonce, err := r.fetchV1ParachainNonce()
	if err != nil {
		return nil, err
	}
	ethNonce, err := r.fetchV1EthereumNonceAtBlock(ctx, executionBlockNumber)
	if err != nil {
		return nil, err
	}
	if ethNonce <= paraNonce {
		return nil, nil
	}

	maxMessages := int(r.writer.MaxWatchedExtrinsics()) - 1
	if maxMessages <= 0 {
		maxMessages = 1
	}
	var nonces []uint64
	for n := paraNonce + 1; n <= ethNonce && len(nonces) < maxMessages; n++ {
		nonces = append(nonces, n)
	}
	events, err := r.findV1Events(ctx, executionBlockNumber, nonces)
	if err != nil {
		return nil, err
	}

	var out []v1Candidate

	next := paraNonce + 1
	for _, ev := range events {
		if len(out) >= maxMessages {
			break
		}
		if ev.Nonce < next || ev.Nonce > ethNonce {
			continue
		}
		if ev.Nonce > next {
			// Gap before the next required nonce — wait for it on a later wake.
			break
		}

		inboundMsg, err := r.makeInboundMessageFromLog(ctx, &ev.Raw)
		if err != nil {
			log.WithError(err).WithField("nonce", ev.Nonce).Warn("[instant-eth] V1 makeInboundMessage failed; stopping contiguous collect")
			break
		}
		source, err := r.getTransactionSenderFromHash(ctx, ev.Raw.TxHash)
		if err != nil {
			log.WithError(err).WithField("nonce", ev.Nonce).Warn("[instant-eth] V1 sender lookup failed; stopping contiguous collect")
			break
		}
		destination, err := r.v1DestinationSS58(ev)
		if err != nil {
			log.WithError(err).WithField("nonce", ev.Nonce).Warn("[instant-eth] V1 destination decode failed; stopping contiguous collect")
			break
		}
		beneficiaries := []string{}
		if destination != "" {
			beneficiaries = []string{destination}
		}
		banned, err := r.ofac.IsBanned(source, beneficiaries)
		if err != nil {
			return nil, fmt.Errorf("OFAC V1: %w", err)
		}
		if banned {
			// Same posture as legacy V1: do not relay past a banned message.
			log.WithField("nonce", ev.Nonce).Warn("[instant-eth] V1 banned address; stopping contiguous collect")
			break
		}

		beaconRoot, err := r.beaconRootForBlock(ctx, ev.Raw.BlockHash, ev.Raw.BlockNumber)
		if err != nil {
			log.WithError(err).WithField("nonce", ev.Nonce).Debug("[instant-eth] V1 beacon root not ready; stopping contiguous collect")
			break
		}
		hdr, err := r.beaconAPI.GetHeaderByBlockRoot(beaconRoot)
		if err != nil {
			log.WithError(err).WithField("nonce", ev.Nonce).Debug("[instant-eth] V1 header fetch failed; stopping contiguous collect")
			break
		}

		var headerUpdate scale.HeaderUpdatePayload
		if hdr.Slot > onchainFinalizedSlot {
			if checkpoint == nil || hdr.Slot > uint64(checkpoint.Payload.FinalizedHeader.Slot) {
				break // needs newer finality than this batch provides
			}
			cp := cacheProofFromUpdate(checkpoint)
			headerUpdate, err = r.beaconHeader.GetHeaderUpdate(beaconRoot, &cp)
		} else {
			headerUpdate, err = r.beaconHeader.GetHeaderUpdateAgainstOnchainFinalized(beaconRoot)
		}
		if err != nil {
			log.WithError(err).WithField("nonce", ev.Nonce).Debug("[instant-eth] V1 execution proof failed; stopping contiguous collect")
			break
		}

		inboundMsg.Proof.ExecutionProof = headerUpdate
		out = append(out, v1Candidate{
			ev:         ev,
			inboundMsg: inboundMsg,
			proof:      headerUpdate,
			source:     source,
		})
		next++
	}

	if len(out) > 0 {
		log.WithField("count", len(out)).Info("[instant-eth] Collected V1 piggyback messages")
	}
	return out, nil
}

// v1DestinationSS58 extracts the V1 payload destination as an SS58 address for OFAC
// (same as legacy ethereum getTransactionDestination). Empty when the command has none.
func (r *Relay) v1DestinationSS58(ev *contractsv1.GatewayOutboundMessageAccepted) (string, error) {
	destination, err := parachain.GetDestination(ev.Payload)
	if err != nil {
		return "", fmt.Errorf("decode V1 destination: %w", err)
	}
	if destination == "" {
		return "", nil
	}
	destinationSS58, err := parachain.SS58Encode(destination, r.config.Sink.SS58Prefix)
	if err != nil {
		return "", fmt.Errorf("ss58 encode: %w", err)
	}
	return destinationSS58, nil
}

func cacheProofFromUpdate(update *scale.Update) cache.Proof {
	return cache.Proof{
		FinalizedBlockRoot: update.FinalizedHeaderBlockRoot,
		BlockRootsTree:     update.BlockRootsTree,
		Slot:               uint64(update.Payload.FinalizedHeader.Slot),
	}
}

// appendV1ToBatch adds EthereumInboundQueue.submit calls (legacy V1 pallet).
func (r *Relay) appendV1ToBatch(extrinsics []string, payloads []interface{}, cands []v1Candidate) ([]string, []interface{}, error) {
	for _, c := range cands {
		c.inboundMsg.Proof.ExecutionProof = c.proof
		submitPayload, err := r.paraconn.GenerateSubmitMessagePayload(c.inboundMsg)
		if err != nil {
			return nil, nil, fmt.Errorf("generate V1 submit payload nonce %d: %w", c.ev.Nonce, err)
		}
		extrinsics = append(extrinsics, "EthereumInboundQueue.submit")
		payloads = append(payloads, submitPayload)
		log.WithField("nonce", c.ev.Nonce).Info("[instant-eth] Including EthereumInboundQueue.submit (V1) in force_batch")
	}
	return extrinsics, payloads, nil
}

func (r *Relay) makeInboundMessageFromLog(ctx context.Context, raw *goethtypes.Log) (*parachain.MessageV3, error) {
	receiptTrie, err := r.headerCache.GetReceiptTrie(ctx, raw.BlockHash)
	if err != nil {
		return nil, err
	}
	return ethereum.MakeMessageFromEvent(raw, receiptTrie)
}

func (r *Relay) getTransactionSenderFromHash(ctx context.Context, txHash common.Hash) (string, error) {
	tx, _, err := r.ethconn.Client().TransactionByHash(ctx, txHash)
	if err != nil {
		return "", err
	}
	sender, err := goethtypes.Sender(goethtypes.LatestSignerForChainID(r.chainID), tx)
	if err != nil {
		return "", fmt.Errorf("retrieve message sender: %w", err)
	}
	return sender.Hex(), nil
}

func (r *Relay) beaconRootForBlock(ctx context.Context, blockHash common.Hash, blockNumber uint64) (common.Hash, error) {
	payloadSlot, hasSlot, err := execution.FetchHeaderSlotNumber(ctx, r.ethconn.Client(), blockHash)
	if err != nil {
		return common.Hash{}, err
	}
	if hasSlot {
		return r.beaconHeader.BeaconRootCommittingToExecutionBlock(payloadSlot, blockHash)
	}
	nextBlockNumber := new(big.Int).SetUint64(blockNumber + 1)
	blockHeader, err := r.ethconn.Client().HeaderByNumber(ctx, nextBlockNumber)
	if err != nil {
		return common.Hash{}, fmt.Errorf("get next block header: %w", err)
	}
	if blockHeader.ParentBeaconRoot == nil {
		return common.Hash{}, fmt.Errorf("block %d has no parentBeaconBlockRoot", nextBlockNumber.Uint64())
	}
	return *blockHeader.ParentBeaconRoot, nil
}
