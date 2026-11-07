package reward

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"math/big"
	"strings"
	"time"

	geth "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	beaconstate "github.com/snowfork/snowbridge/relayer/relays/beacon-state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config          *Config
	keypair         *sr25519.Keypair
	paraconn        *parachain.Connection
	ethconn         *ethereum.Connection
	gatewayContract *contracts.Gateway
	beaconHeader    *header.Header
	headerCache     *ethereum.HeaderCache
	writer          *parachain.ParachainWriter
	skippedNonces   map[uint64]bool
	deadNonces      *error_tracking.DeadNonceStore
}

func NewRelay(
	config *Config,
	keypair *sr25519.Keypair,
) *Relay {
	return &Relay{
		config:        config,
		keypair:       keypair,
		skippedNonces: make(map[uint64]bool),
	}
}

func (r *Relay) isSkippedNonce(nonce uint64) bool {
	if r.skippedNonces[nonce] {
		return true
	}
	if r.deadNonces != nil && r.deadNonces.IsDead(nonce) {
		return true
	}
	return false
}

// markSkipped permanently skips nonce for this process and, when a path is
// configured, persists it so restarts keep skipping it.
func (r *Relay) markSkipped(nonce uint64, reason string) {
	r.skippedNonces[nonce] = true
	if r.deadNonces == nil {
		return
	}
	if err := r.deadNonces.MarkDead(nonce, reason); err != nil {
		log.WithError(err).WithField("nonce", nonce).Error("failed to persist dead reward nonce")
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	deadNonces, err := error_tracking.NewDeadNonceStore(r.config.DeadNoncesPath)
	if err != nil {
		return fmt.Errorf("init dead-nonce store: %w", err)
	}
	r.deadNonces = deadNonces

	paraconn := parachain.NewConnection(r.config.Sink.Parachain.ParseEndpoints(), r.keypair.AsKeyringPair())
	ethconn := ethereum.NewConnection(&r.config.Source.Ethereum, nil)

	err = paraconn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(r.config.Sink.Parachain.HeartbeatSecs))
	if err != nil {
		return err
	}
	r.paraconn = paraconn

	err = ethconn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(r.config.Source.Ethereum.HeartbeatSecs))
	if err != nil {
		return err
	}
	r.ethconn = ethconn

	r.writer = parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
	)

	err = r.writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	headerCache, err := ethereum.NewHeaderBlockCache(
		&ethereum.DefaultBlockLoader{Conn: ethconn},
	)
	if err != nil {
		return err
	}
	r.headerCache = headerCache

	address := common.HexToAddress(r.config.Source.Contracts.Gateway)
	contract, err := contracts.NewGateway(address, ethconn.Client())
	if err != nil {
		return err
	}
	r.gatewayContract = contract

	p := protocol.New(r.config.Source.Beacon.Spec, r.config.Sink.Parachain.HeaderRedundancy)

	beaconAPI := api.NewBeaconClient(r.config.Source.Beacon.ParseEndpoints())

	stateServiceClient := beaconstate.NewClient(r.config.Source.Beacon.StateServiceEndpoint)
	log.WithField("endpoint", r.config.Source.Beacon.StateServiceEndpoint).Info("Using beacon state service for proof generation")

	beaconHeader := header.New(
		r.writer,
		beaconAPI,
		r.config.Source.Beacon.Spec,
		p,
		0, // setting is not used in the reward relay
		stateServiceClient,
	)
	// From Gloas the execution header is no longer in the beacon block, so the proof path
	// reads it from the execution node this relay is already connected to.
	beaconHeader = beaconHeader.WithExecutionClient(r.ethconn.Client())
	r.beaconHeader = &beaconHeader

	var fetchInterval time.Duration
	if r.config.FetchInterval == 0 {
		fetchInterval = 60 * time.Second
	} else {
		fetchInterval = time.Duration(r.config.FetchInterval) * time.Second
	}

	// Poll immediately on startup, then on every fetchInterval.
	// Waiting a full interval first delayed recovery (e.g. 300s in prod) after restarts.
	ticker := time.NewTicker(fetchInterval)
	defer ticker.Stop()

	for {
		err = r.pollOnce(ctx)
		if err != nil {
			if error_tracking.IsTransientError(err) {
				log.WithError(err).Warn("Transient error in reward relay, will retry next interval")
				r.reconnectAndRebind(ctx)
			} else {
				return err
			}
		}

		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
		}
	}
}

const staleBlockThreshold = 100_000 // ~2 weeks of Ethereum blocks

func (r *Relay) pollOnce(ctx context.Context) error {
	log.Info("Polling for reward claims")
	orders, err := r.findOrderUndelivered(ctx)
	if err != nil {
		return fmt.Errorf("find undelivered order: %w", err)
	}
	rewardAddress, err := util.HexStringTo32Bytes(r.config.RewardAddress)
	if err != nil {
		return fmt.Errorf("convert to reward address: %w", err)
	}

	currentBlock, err := r.ethconn.Client().BlockNumber(ctx)
	if err != nil {
		return fmt.Errorf("get current block number: %w", err)
	}

	for _, order := range orders {
		nonce := order.Nonce
		if r.isSkippedNonce(nonce) {
			continue
		}
		event, err := r.findEvent(ctx, nonce)
		if err != nil {
			return fmt.Errorf("find event GatewayInboundMessageDispatched: %w", err)
		}
		if event.RewardAddress != rewardAddress {
			r.markSkipped(nonce, "order is not from the current relayer")
			log.WithField("nonce", nonce).Debug("order is not from the current relayer, skipping permanently")
			continue
		}
		if currentBlock-event.Raw.BlockNumber > staleBlockThreshold {
			r.markSkipped(nonce, "stale reward nonce, too old to finalize")
			log.WithFields(log.Fields{
				"nonce":        nonce,
				"eventBlock":   event.Raw.BlockNumber,
				"currentBlock": currentBlock,
			}).Info("Skipping stale reward nonce, too old to finalize")
			continue
		}
		err = r.doSubmit(ctx, event)
		if err != nil {
			return fmt.Errorf("submit delivery proof for GatewayInboundMessageDispatched: %w", err)
		}
	}
	return nil
}

func (r *Relay) reconnectAndRebind(ctx context.Context) {
	if err := r.ethconn.Reconnect(ctx); err != nil {
		log.WithError(err).Error("Ethereum reconnection failed")
	} else {
		if err := r.rebindContracts(); err != nil {
			log.WithError(err).Error("Failed to rebind contracts after reconnect")
		}
	}
	if err := r.paraconn.Reconnect(); err != nil {
		log.WithError(err).Error("Parachain reconnection failed")
	}
}

func (r *Relay) rebindContracts() error {
	address := common.HexToAddress(r.config.Source.Contracts.Gateway)
	contract, err := contracts.NewGateway(address, r.ethconn.Client())
	if err != nil {
		return fmt.Errorf("rebind gateway contract: %w", err)
	}
	r.gatewayContract = contract
	return nil
}

func (r *Relay) isNonceRelayed(ctx context.Context, nonce uint64) (bool, error) {
	options := bind.CallOpts{
		Pending: true,
		Context: ctx,
	}
	isRelayed, err := r.gatewayContract.V2IsDispatched(&options, nonce)
	if err != nil {
		return isRelayed, fmt.Errorf("check nonce from gateway contract: %w", err)
	}
	return isRelayed, nil
}

func (r *Relay) findOrderUndelivered(
	ctx context.Context,
) ([]*parachain.PendingOrder, error) {
	storageKey := types.NewStorageKey(types.CreateStorageKeyPrefix("EthereumOutboundQueueV2", "PendingOrders"))
	keys, err := r.paraconn.API().RPC.State.GetKeysLatest(storageKey)
	if err != nil {
		return nil, fmt.Errorf("fetch nonces from PendingOrders start with key '%v': %w", storageKey, err)
	}
	var undeliveredOrders []*parachain.PendingOrder
	for _, key := range keys {
		var undeliveredOrder parachain.PendingOrder
		value, err := r.paraconn.API().RPC.State.GetStorageRawLatest(key)
		if err != nil {
			return nil, fmt.Errorf("fetch value of pendingOrder with key '%v': %w", key, err)
		}
		decoder := scale.NewDecoder(bytes.NewReader(*value))
		err = decoder.Decode(&undeliveredOrder)
		if err != nil {
			return nil, fmt.Errorf("decode order error: %w", err)
		}
		nonce := uint64(undeliveredOrder.Nonce)
		if r.isSkippedNonce(nonce) {
			continue
		}
		isRelayed, err := r.isNonceRelayed(ctx, nonce)
		if err != nil {
			return nil, fmt.Errorf("check nonce relayed: %w", err)
		}
		if isRelayed {
			log.WithFields(log.Fields{
				"nonce": nonce,
			}).Debug("Relayed but not delivered to BH")
			undeliveredOrders = append(undeliveredOrders, &undeliveredOrder)
		}
	}
	return undeliveredOrders, nil
}

func (relay *Relay) findEvent(
	ctx context.Context,
	nonce uint64,
) (*contracts.GatewayInboundMessageDispatched, error) {

	var event *contracts.GatewayInboundMessageDispatched

	blockNumber, err := relay.ethconn.Client().BlockNumber(ctx)
	if err != nil {
		return event, fmt.Errorf("get last block number: %w", err)
	}

	err = ethereum.WalkLogFilterPages(ctx, blockNumber, func(opts *bind.FilterOpts) (bool, error) {
		iter, err := relay.gatewayContract.FilterInboundMessageDispatched(opts, []uint64{nonce})
		if err != nil {
			return false, fmt.Errorf("iter dispatch event: %w", err)
		}

		done := false
		for {
			more := iter.Next()
			if !more {
				if err := iter.Error(); err != nil {
					return false, fmt.Errorf("iter dispatch event: %w", err)
				}
				break
			}
			if iter.Event.Nonce == nonce {
				event = iter.Event
				done = true
				break
			}
		}
		if done {
			iter.Close()
		}
		return done, nil
	})
	if err != nil {
		return event, err
	}

	return event, nil
}

func (relay *Relay) makeInboundMessage(
	ctx context.Context,
	headerCache *ethereum.HeaderCache,
	event *contracts.GatewayInboundMessageDispatched,
) (*parachain.MessageV3, error) {
	receiptTrie, err := headerCache.GetReceiptTrie(ctx, event.Raw.BlockHash)
	if err != nil {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to get receipt trie for event")
		return nil, err
	}

	msg, err := ethereum.MakeMessageFromEvent(&event.Raw, receiptTrie)
	if err != nil {
		log.WithFields(logrus.Fields{
			"address":     event.Raw.Address.Hex(),
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to generate message from ethereum event")
		return nil, err
	}

	log.WithFields(logrus.Fields{
		"blockHash":   event.Raw.BlockHash.Hex(),
		"blockNumber": event.Raw.BlockNumber,
		"txHash":      event.Raw.TxHash.Hex(),
	}).Info("found message")

	return msg, nil
}

func (relay *Relay) doSubmit(ctx context.Context, ev *contracts.GatewayInboundMessageDispatched) error {
	inboundMsg, err := relay.makeInboundMessage(ctx, relay.headerCache, ev)
	if err != nil {
		return fmt.Errorf("make outgoing message: %w", err)
	}

	logger := log.WithFields(log.Fields{
		"ethNonce":    ev.Nonce,
		"msgNonce":    ev.Nonce,
		"address":     ev.Raw.Address.Hex(),
		"blockHash":   ev.Raw.BlockHash.Hex(),
		"blockNumber": ev.Raw.BlockNumber,
		"txHash":      ev.Raw.TxHash.Hex(),
		"txIndex":     ev.Raw.TxIndex,
	})

	nextBlockNumber := new(big.Int).SetUint64(ev.Raw.BlockNumber + 1)

	blockHeader, err := relay.ethconn.Client().HeaderByNumber(ctx, nextBlockNumber)
	if err != nil {
		// The delivery landed in the head block, so block+1 does not exist yet. Expected
		// on a fresh delivery; retry next poll rather than crashing the relayer.
		if isBlockNotMinedError(err) {
			logger.Info("next block is not mined yet")
			return nil
		}
		return fmt.Errorf("get block header: %w", err)
	}
	if blockHeader == nil {
		logger.Info("next block is not mined yet")
		return nil
	}

	proof, err := relay.beaconHeader.FetchExecutionProof(*blockHeader.ParentBeaconRoot, false)
	// Only treat real light-client lag as soft "not finalized". Zero-value proofs
	// used to match ExecutionBranch == nil on *any* error and infinite-looped as
	// "not finalized" (e.g. lodestar 404 after pruneHistory dropped the root).
	if errors.Is(err, header.ErrBeaconHeaderNotFinalized) {
		logger.Info("event block is not finalized yet")
		return nil
	}
	if err != nil {
		if isUnrecoverableBeaconCheckpointError(err) {
			relay.markSkipped(ev.Nonce, "no on-chain beacon checkpoint covers this slot: "+err.Error())
			logger.WithError(err).Error("permanently skipping reward claim: no on-chain beacon checkpoint covers this slot")
			return nil
		}
		if isBeaconHistoryUnavailableError(err) {
			// Soft-skip so other nonces can still claim; retry next poll if archive CL recovers.
			logger.WithError(err).Warn("cannot build execution proof for reward claim (beacon history unavailable); will retry")
			return nil
		}
		return fmt.Errorf("fetch execution header proof: %w", err)
	}
	if len(proof.HeaderPayload.ExecutionBranch) == 0 {
		logger.Warn("execution proof missing execution branch; treating as not ready")
		return nil
	}

	inboundMsg.Proof.ExecutionProof = proof.HeaderPayload

	log.WithFields(logrus.Fields{
		"EventLog": inboundMsg.EventLog,
		"Proof":    inboundMsg.Proof,
	}).Debug("Generated message from Ethereum log")

	submitPayload, err := relay.paraconn.GenerateSubmitMessagePayload(inboundMsg)
	if err != nil {
		return fmt.Errorf("generate submit message payload: %w", err)
	}

	err = relay.writer.WriteToParachainAndWatch(ctx, "EthereumOutboundQueueV2.submit_delivery_receipt", submitPayload)
	if err != nil {
		return fmt.Errorf("submit message to outbound queue v2: %w", err)
	}

	logger.Info("v2 inbound message executed successfully")

	return nil
}

// isBeaconHistoryUnavailableError matches beacon/CL responses when the header or
// intermediate state is gone from (or not yet present on) the configured endpoints.
// Multi-endpoint clients should retry against an archive CL for these.
func isBeaconHistoryUnavailableError(err error) bool {
	if err == nil {
		return false
	}
	if errors.Is(err, api.ErrNotFound) {
		return true
	}
	s := strings.ToLower(err.Error())
	return strings.Contains(s, "not found") ||
		strings.Contains(s, "block not found") ||
		strings.Contains(s, "unable to find a relevant on-chain header") ||
		strings.Contains(s, "find checkpoint on-chain") ||
		strings.Contains(s, "get header update against on-chain finalized")
}

// isBlockNotMinedError matches an execution-client miss for a block that does not exist
// yet. go-ethereum returns ethereum.NotFound; other clients only set the message.
func isBlockNotMinedError(err error) bool {
	if err == nil {
		return false
	}
	if errors.Is(err, geth.NotFound) {
		return true
	}
	return strings.Contains(strings.ToLower(err.Error()), "not found")
}

// isUnrecoverableBeaconCheckpointError is for proofs that cannot be built because
// BridgeHub no longer stores a finalized header within the historical-root window.
func isUnrecoverableBeaconCheckpointError(err error) bool {
	if err == nil {
		return false
	}
	return strings.Contains(strings.ToLower(err.Error()), "no checkpoint on chain for slot")
}
