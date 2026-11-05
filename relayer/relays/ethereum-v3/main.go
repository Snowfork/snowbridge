package executionV3

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"os"
	"slices"
	"sort"
	"strconv"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	ethtypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	contractsv1 "github.com/snowfork/snowbridge/relayer/contracts/v1"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/ofac"
	beaconstate "github.com/snowfork/snowbridge/relayer/relays/beacon-state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/execution"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
	"github.com/snowfork/snowbridge/relayer/relays/gasestimator"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"github.com/snowfork/snowbridge/relayer/relays/xcmv5"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config                  *Config
	keypair                 *sr25519.Keypair
	paraconn                *parachain.Connection
	ethconn                 *ethereum.Connection
	gatewayContract         *contracts.Gateway
	gatewayContractV1       *contractsv1.Gateway
	beaconHeader            *header.Header
	beaconAPI               *api.BeaconClient
	writer                  *parachain.ParachainWriter
	headerCache             *ethereum.HeaderCache
	ofac                    *ofac.OFAC
	chainID                 *big.Int
	deadNonces              *error_tracking.DeadNonceStore
	gasEstimator            *gasestimator.Estimator
	minMessageRelayerFee    *big.Int
	serviceFeeRecipients    [][32]byte
	latestFinalizedSlot     uint64
	latestOnchainBeaconSlot uint64
	mu                      sync.Mutex
	// nonceBlocksMu guards the per-nonce emitting-block caches shared by the tip
	// and catch-up scans.
	nonceBlocksMu sync.Mutex
	v2NonceBlocks map[uint64]uint64
	v1NonceBlocks map[uint64]uint64
	// deferredMu guards deferredV2, the retry time of each V2 nonce left out of
	// candidate selection by the tip and catch-up scans.
	deferredMu sync.Mutex
	deferredV2 map[uint64]time.Time
	// ethMu serializes gateway/chainID pointer swaps in reconnectAndRebind against
	// tip + catch-up scans that read those fields (and share eth/para connections).
	ethMu   sync.RWMutex
	trigger chan wakeKind
}

// wakeKind identifies why the worker was woken. It controls whether a scan may
// attempt EthereumBeaconClient.submit (beacon-capable) or is restricted to
// message-only submits against the existing on-chain light client.
type wakeKind int

const (
	wakeStartup wakeKind = iota
	wakeSSE
	wakeOnchainLC
	// wakeMessagePoll is a periodic message-only scan (parachain-v3 direct-relay
	// analogue): deliver pending V1/V2 that already prove against on-chain LC
	// without waiting for the next SSE / LC header move.
	wakeMessagePoll
)

func (k wakeKind) String() string {
	switch k {
	case wakeStartup:
		return "startup"
	case wakeSSE:
		return "sse"
	case wakeOnchainLC:
		return "onchain-lc"
	case wakeMessagePoll:
		return "message-poll"
	default:
		return fmt.Sprintf("wake(%d)", int(k))
	}
}

// allowBeaconSubmit is false for message-only wakes: on-chain LC is already far
// enough (or unchanged); the worker only proves/submits messages against it.
func (k wakeKind) allowBeaconSubmit() bool {
	return k == wakeStartup || k == wakeSSE
}

// messagePollInterval is how often the worker re-scans for pending V1/V2 that
// the existing on-chain beacon LC can already verify (same role as parachain-v3
// catch-up tryDirectRelayAnchoredMessages). Override with ETHEREUM_V3_MESSAGE_POLL_SECS
// (0 disables).
var messagePollInterval = messagePollIntervalFromEnv()

func messagePollIntervalFromEnv() time.Duration {
	const defaultSecs = 120
	raw := os.Getenv("ETHEREUM_V3_MESSAGE_POLL_SECS")
	if raw == "" {
		return defaultSecs * time.Second
	}
	secs, err := strconv.ParseUint(raw, 10, 64)
	if err != nil {
		log.WithError(err).WithField("value", raw).Warn("[instant-eth] invalid ETHEREUM_V3_MESSAGE_POLL_SECS; using default")
		return defaultSecs * time.Second
	}
	if secs == 0 {
		return 0
	}
	return time.Duration(secs) * time.Second
}

// finalityUpdatePollInterval / finalityUpdatePollTimeout: after SSE, Lodestar's
// finality_update often lags the finalized_checkpoint by a few seconds. Short-poll
// instead of giving up on the first lagging response so we can still force_batch
// beacon+messages before the free-ride race.
const finalityUpdatePollInterval = 1 * time.Second
const finalityUpdatePollTimeout = 15 * time.Second

// finalityUpdateCoversRequired reports whether an LC finality_update can advance
// on-chain state and prove candidates that need consensus up to requiredSlot.
func finalityUpdateCoversRequired(updateSlot, onchainSlot, requiredSlot uint64) bool {
	return updateSlot > onchainSlot && updateSlot >= requiredSlot
}

func mergeWake(a, b wakeKind) wakeKind {
	if !a.allowBeaconSubmit() && !b.allowBeaconSubmit() {
		// Keep on-chain-lc label when either side was LC-driven; else message-poll.
		if a == wakeOnchainLC || b == wakeOnchainLC {
			return wakeOnchainLC
		}
		return wakeMessagePoll
	}
	// Prefer beacon-capable wakes (SSE/startup) over message-only wakes.
	if a == wakeStartup || b == wakeStartup {
		return wakeStartup
	}
	return wakeSSE
}

func (r *Relay) isIgnoredNonce(nonce uint64) bool {
	if r == nil || r.config == nil {
		return false
	}
	return slices.Contains(r.config.IgnoreNonces, nonce)
}

func NewRelay(
	config *Config,
	keypair *sr25519.Keypair,
) *Relay {
	// Per-message reward floor (wei) on RelayerFee + decoded UI/service fee.
	// Empty/invalid -> 0, i.e. no filter.
	minMessageRelayerFee := new(big.Int)
	if s := config.MinMessageRelayerFeeWei; s != "" {
		if v, ok := new(big.Int).SetString(s, 10); ok {
			minMessageRelayerFee = v
		}
	}
	recipients, err := xcmv5.ParseRecipients(config.GasEstimation.ServiceFeeRecipients)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] invalid service-fee-recipients; treating as empty")
		recipients = nil
	}
	return &Relay{
		config:               config,
		keypair:              keypair,
		gasEstimator:         gasestimator.New(config.GasEstimation),
		minMessageRelayerFee: minMessageRelayerFee,
		serviceFeeRecipients: recipients,
		v2NonceBlocks:        make(map[uint64]uint64),
		v1NonceBlocks:        make(map[uint64]uint64),
		deferredV2:           make(map[uint64]time.Time),
		trigger:              make(chan wakeKind, 1),
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
	r.writer.SetTip(r.config.Tip)

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
	if r.config.V1Enabled() {
		v1Contract, err := contractsv1.NewGateway(address, ethconn.Client())
		if err != nil {
			return fmt.Errorf("bind V1 gateway: %w", err)
		}
		r.gatewayContractV1 = v1Contract
	}

	p := protocol.New(r.config.Source.Beacon.Spec, r.config.Sink.Parachain.HeaderRedundancy)

	r.ofac = ofac.New(r.config.OFAC.Enabled, r.config.OFAC.ApiKey)

	beaconAPI := api.NewBeaconClient(r.config.Source.Beacon.ParseEndpoints())
	r.beaconAPI = beaconAPI

	stateServiceClient := beaconstate.NewClient(r.config.Source.Beacon.StateServiceEndpoint)
	log.WithField("endpoint", r.config.Source.Beacon.StateServiceEndpoint).Info("Using beacon state service for proof generation")

	// Gloas: execution header comes from the execution node (not the beacon block).
	// Catch-up LC: bare submit when solo; ForceBatch when piggybacking V1/V2 (replaces standalone beacon).
	beaconHeader := header.New(
		r.writer,
		beaconAPI,
		r.config.Source.Beacon.Spec,
		p,
		0, // setting is not used in the execution relay
		stateServiceClient,
	).WithExecutionClient(r.ethconn.Client()).WithFinalizedSubmitter(r.submitBeaconCatchUp)
	r.beaconHeader = &beaconHeader

	r.chainID, err = r.ethconn.Client().NetworkID(ctx)
	if err != nil {
		return err
	}

	log.WithFields(log.Fields{
		"chainId":              r.chainID,
		"v1Enabled":            r.config.V1Enabled(),
		"minRelayerFeeWei":     r.minMessageRelayerFee.String(),
		"serviceFeeRecipients": len(r.serviceFeeRecipients),
	}).Info("relayer config")

	log.WithField("gasEstimation", r.gasEstimator.Enabled()).Info("[instant-eth] ethereum-v3: tip + beacon catch-up; V2 gas-estimator optional; V1 tip piggyback always")

	// Seed the on-chain LC slot so the initial SubscribeStorage notification
	// (current value) does not spuriously wake the worker.
	if onchain, err := r.writer.GetLastFinalizedHeaderStateAtBestBlock(); err != nil {
		log.WithError(err).Warn("[instant-eth] Failed to seed on-chain beacon slot; BH LC watch will seed on first observed advance")
	} else {
		r.mu.Lock()
		r.latestOnchainBeaconSlot = onchain.BeaconSlot
		r.mu.Unlock()
		log.WithField("onchainBeaconSlot", onchain.BeaconSlot).Info("[instant-eth] Seeded on-chain beacon slot for BH LC watch")
	}

	eg.Go(func() error {
		var backoff = 1 * time.Second
		for {
			select {
			case <-ctx.Done():
				return nil
			default:
			}

			log.Info("[instant-eth] Connecting to Lodestar SSE event stream")
			eventsCh, errCh, err := r.beaconAPI.SubscribeFinalizedCheckpoint(ctx)
			if err != nil {
				log.WithError(err).Warnf("[instant-eth] Failed to subscribe to SSE stream, retrying in %v", backoff)
				select {
				case <-ctx.Done():
					return nil
				case <-time.After(backoff):
					backoff = minDuration(backoff*2, 30*time.Second)
					continue
				}
			}

			backoff = 1 * time.Second

			err = r.consumeSSEEvents(ctx, eventsCh, errCh)
			if err != nil {
				if errors.Is(err, context.Canceled) {
					return nil
				}
				log.WithError(err).Warnf("[instant-eth] SSE stream closed/error, reconnecting in %v", backoff)
				select {
				case <-ctx.Done():
					return nil
				case <-time.After(backoff):
					backoff = minDuration(backoff*2, 30*time.Second)
				}
			}
		}
	})

	eg.Go(func() error {
		return r.watchOnchainBeaconClient(ctx)
	})

	if err := r.beaconHeader.Sync(ctx, eg); err != nil {
		return fmt.Errorf("start beacon catch-up sync: %w", err)
	}
	log.Info("[instant-eth] beacon catch-up Header.Sync started (LC ± messages)")

	// Periodic message-only poll: deliver pending V1/V2 already verifiable against
	// on-chain LC without waiting for the next SSE / LC header move (parachain-v3
	// tryDirectRelayAnchoredMessages analogue). Coalesces onto the single worker.
	if messagePollInterval > 0 {
		eg.Go(func() error {
			ticker := time.NewTicker(messagePollInterval)
			defer ticker.Stop()
			log.WithField("interval", messagePollInterval.String()).Info("[instant-eth] message-only poll ticker started")
			for {
				select {
				case <-ctx.Done():
					return nil
				case <-ticker.C:
					log.Debug("[instant-eth] message-only poll tick")
					r.sendWake(wakeMessagePoll)
				}
			}
		})
	}

	eg.Go(func() error {
		log.Info("[instant-eth] Triggering initial startup sync")
		if err := r.handleLoopError(ctx, r.runBatchRelay(ctx, wakeStartup), "initial startup sync"); err != nil {
			return err
		}

		for {
			select {
			case <-ctx.Done():
				return nil
			case kind := <-r.trigger:
				if err := r.handleLoopError(ctx, r.runBatchRelay(ctx, kind), "worker batch relay"); err != nil {
					return err
				}
			}
		}
	})

	return nil
}

func (r *Relay) runBatchRelay(ctx context.Context, kind wakeKind) error {
	checkpoint, err := r.beaconAPI.GetFinalizedCheckpoint()
	if err != nil {
		return fmt.Errorf("fetch finalized checkpoint: %w", err)
	}
	return r.doScanAndBatchRelay(ctx, checkpoint, kind)
}

// handleLoopError centralizes the fast-path error policy: transient errors are
// logged, self-healed (reconnect) and swallowed so the loop keeps running;
// persistent errors are wrapped and returned so the errgroup terminates and the
// supervisor restarts the process. what names the step for logging/wrapping.
func (r *Relay) handleLoopError(ctx context.Context, err error, what string) error {
	if err == nil {
		return nil
	}
	if error_tracking.IsTransientError(err) {
		log.WithError(err).Warnf("[instant-eth] transient error during %s; self-healing and retrying", what)
		r.reconnectAndRebind(ctx)
		return nil
	}
	return fmt.Errorf("%s: %w", what, err)
}

func (r *Relay) consumeSSEEvents(ctx context.Context, eventsCh <-chan api.FinalizedCheckpointEvent, errCh <-chan error) error {
	for {
		select {
		case <-ctx.Done():
			return nil
		case err, ok := <-errCh:
			if !ok {
				return nil
			}
			return err
		case event, ok := <-eventsCh:
			if !ok {
				return nil
			}
			log.WithFields(log.Fields{
				"epoch": event.Epoch,
				"block": event.Block,
			}).Info("[instant-eth] Received finalized checkpoint SSE event")

			blockRoot := common.HexToHash(event.Block)
			header, err := r.beaconAPI.GetHeaderByBlockRoot(blockRoot)
			if err != nil {
				if error_tracking.IsTransientError(err) {
					log.WithError(err).WithField("block", event.Block).Warn("[instant-eth] Transient error fetching header for SSE finalized checkpoint; skipping event")
					continue
				}
				return fmt.Errorf("fetch header for SSE finalized checkpoint block %s: %w", event.Block, err)
			}

			r.triggerIfFinalizedSlotAdvanced(header.Slot)
		}
	}
}

func minDuration(a, b time.Duration) time.Duration {
	if a < b {
		return a
	}
	return b
}

// sendWake coalesces fast-path wakes onto the buffered trigger channel. If a wake
// is already pending, mergeWake keeps beacon-capable intent when either side has it.
func (r *Relay) sendWake(kind wakeKind) {
	select {
	case r.trigger <- kind:
		log.WithField("wake", kind.String()).Info("[instant-eth] Sent trigger to worker queue")
		return
	default:
	}

	select {
	case pending := <-r.trigger:
		merged := mergeWake(pending, kind)
		select {
		case r.trigger <- merged:
			log.WithFields(log.Fields{
				"pending": pending.String(),
				"new":     kind.String(),
				"merged":  merged.String(),
			}).Debug("[instant-eth] Worker queue full; coalesced wake")
		default:
			log.WithField("wake", kind.String()).Debug("[instant-eth] Worker queue full after coalesce race; dropping wake")
		}
	default:
		select {
		case r.trigger <- kind:
			log.WithField("wake", kind.String()).Info("[instant-eth] Sent trigger to worker queue")
		default:
			log.WithField("wake", kind.String()).Debug("[instant-eth] Worker queue full, coalescing trigger")
		}
	}
}

// triggerIfFinalizedSlotAdvanced coalesces fast-path wakes on eth finalized
// checkpoint advancement. Pass the checkpoint header slot (SSE event block root
// or GetFinalizedCheckpoint), not light-client finality_update: we wake as soon
// as the eth checkpoint advances so scan can start while waitForFinalityUpdateAhead
// short-polls LC and force_batches beacon+messages once ready. Gating wakes on a
// lagging finality_update slot would delay or drop the SSE wake entirely.
func (r *Relay) triggerIfFinalizedSlotAdvanced(slot uint64) {
	r.mu.Lock()
	if slot <= r.latestFinalizedSlot {
		r.mu.Unlock()
		log.WithField("slot", slot).Debug("[instant-eth] Slot did not advance past last processed slot; ignoring trigger")
		return
	}
	r.latestFinalizedSlot = slot
	r.mu.Unlock()

	r.sendWake(wakeSSE)
}

// triggerIfOnchainBeaconSlotAdvanced coalesces fast-path triggers when Bridge Hub's
// EthereumBeaconClient finalized slot advances (often from a competing relayer).
// Tracked separately from latestFinalizedSlot so an SSE scan that already saw the
// eth checkpoint still wakes the worker once the on-chain LC catches up.
func (r *Relay) triggerIfOnchainBeaconSlotAdvanced(slot uint64) {
	r.mu.Lock()
	if slot <= r.latestOnchainBeaconSlot {
		r.mu.Unlock()
		log.WithField("onchainBeaconSlot", slot).Debug("[instant-eth] On-chain beacon slot did not advance; ignoring trigger")
		return
	}
	r.latestOnchainBeaconSlot = slot
	r.mu.Unlock()

	r.sendWake(wakeOnchainLC)
}

// waitForFinalityUpdateAhead polls Lodestar finality_update until it is strictly ahead
// of on-chain and covers requiredSlot, or until finalityUpdatePollTimeout. A lagging
// first response is common for ~1–15s after SSE finalized_checkpoint; returning that
// on-chain-equal update immediately skips the force_batch race. Transient fetch errors
// are retried until timeout; the last update (or zero-slot on total failure) is returned
// so the caller can skip beacon this round (dedicated beacon relayer / BH LC watch recover).
func (r *Relay) waitForFinalityUpdateAhead(ctx context.Context, onchainSlot, requiredSlot uint64) (scale.Update, uint64, error) {
	deadline := time.Now().Add(finalityUpdatePollTimeout)
	var lastUpdate scale.Update
	var lastSlot uint64
	var lastErr error
	attempt := 0

	for {
		attempt++
		update, err := r.beaconHeader.GetLatestFinalizedUpdate()
		if err != nil {
			lastErr = err
			log.WithError(err).WithFields(log.Fields{
				"attempt":      attempt,
				"onchainSlot":  onchainSlot,
				"requiredSlot": requiredSlot,
			}).Warn("[instant-eth] LC finality_update fetch failed; retrying short-poll")
		} else {
			lastUpdate = update
			lastSlot = uint64(update.Payload.FinalizedHeader.Slot)
			lastErr = nil
			if finalityUpdateCoversRequired(lastSlot, onchainSlot, requiredSlot) {
				log.WithFields(log.Fields{
					"updateSlot":   lastSlot,
					"onchainSlot":  onchainSlot,
					"requiredSlot": requiredSlot,
					"attempt":      attempt,
				}).Info("[instant-eth] LC finality_update ready for beacon submit")
				return lastUpdate, lastSlot, nil
			}
			log.WithFields(log.Fields{
				"updateSlot":   lastSlot,
				"onchainSlot":  onchainSlot,
				"requiredSlot": requiredSlot,
				"attempt":      attempt,
			}).Info("[instant-eth] LC finality_update not ready yet; short-polling")
		}

		if !time.Now().Before(deadline) {
			break
		}

		timer := time.NewTimer(finalityUpdatePollInterval)
		select {
		case <-ctx.Done():
			timer.Stop()
			return scale.Update{}, 0, ctx.Err()
		case <-timer.C:
		}
		if !time.Now().Before(deadline) {
			break
		}
	}

	if lastSlot == 0 && lastErr != nil {
		return scale.Update{}, 0, lastErr
	}
	log.WithFields(log.Fields{
		"updateSlot":   lastSlot,
		"onchainSlot":  onchainSlot,
		"requiredSlot": requiredSlot,
		"timeout":      finalityUpdatePollTimeout.String(),
		"attempts":     attempt,
	}).Info("[instant-eth] LC finality_update short-poll timed out")
	return lastUpdate, lastSlot, nil
}

// watchOnchainBeaconClient subscribes to EthereumBeaconClient::LatestFinalizedBlockRoot
// on Bridge Hub and wakes the worker whenever the on-chain light-client slot advances.
func (r *Relay) watchOnchainBeaconClient(ctx context.Context) error {
	backoff := time.Second
	for {
		select {
		case <-ctx.Done():
			return nil
		default:
		}

		key, err := types.CreateStorageKey(r.paraconn.Metadata(), "EthereumBeaconClient", "LatestFinalizedBlockRoot", nil, nil)
		if err != nil {
			log.WithError(err).Warnf("[instant-eth] Failed to create LatestFinalizedBlockRoot storage key, retrying in %v", backoff)
			select {
			case <-ctx.Done():
				return nil
			case <-time.After(backoff):
				backoff = minDuration(backoff*2, 30*time.Second)
				continue
			}
		}

		log.Info("[instant-eth] Subscribing to Bridge Hub EthereumBeaconClient::LatestFinalizedBlockRoot")
		sub, err := r.paraconn.API().RPC.State.SubscribeStorageRaw([]types.StorageKey{key})
		if err != nil {
			log.WithError(err).Warnf("[instant-eth] Failed to subscribe to BH LC storage, retrying in %v", backoff)
			select {
			case <-ctx.Done():
				return nil
			case <-time.After(backoff):
				backoff = minDuration(backoff*2, 30*time.Second)
				continue
			}
		}
		backoff = time.Second

		err = r.consumeOnchainBeaconClientEvents(ctx, sub.Chan(), sub.Err())
		sub.Unsubscribe()
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			log.WithError(err).Warnf("[instant-eth] BH LC storage subscription closed, reconnecting in %v", backoff)
			select {
			case <-ctx.Done():
				return nil
			case <-time.After(backoff):
				backoff = minDuration(backoff*2, 30*time.Second)
			}
		}
	}
}

func (r *Relay) consumeOnchainBeaconClientEvents(
	ctx context.Context,
	changes <-chan types.StorageChangeSet,
	errs <-chan error,
) error {
	for {
		select {
		case <-ctx.Done():
			return context.Canceled
		case err, ok := <-errs:
			if !ok {
				return fmt.Errorf("BH LC subscription error channel closed")
			}
			if err != nil {
				return err
			}
			return fmt.Errorf("BH LC subscription ended")
		case change, ok := <-changes:
			if !ok {
				return fmt.Errorf("BH LC subscription channel closed")
			}
			onchain, err := r.writer.GetLastFinalizedHeaderStateAtBestBlock()
			if err != nil {
				if error_tracking.IsTransientError(err) {
					log.WithError(err).WithField("block", change.Block.Hex()).Warn("[instant-eth] Transient error reading on-chain beacon state after LC storage change; skipping")
					continue
				}
				return fmt.Errorf("read on-chain beacon state after LC storage change: %w", err)
			}
			r.triggerIfOnchainBeaconSlotAdvanced(onchain.BeaconSlot)
		}
	}
}

func (r *Relay) reconnectAndRebind(ctx context.Context) {
	// Exclusive lock: wait out in-flight tip/catch-up scans that hold ethMu.RLock
	// before swapping contract bindings or reconnecting shared RPC clients.
	r.ethMu.Lock()
	defer r.ethMu.Unlock()

	if err := r.ethconn.Reconnect(ctx); err != nil {
		log.WithError(err).Error("Ethereum reconnection failed")
	} else {
		if err := r.rebindContractsLocked(ctx); err != nil {
			log.WithError(err).Error("Failed to rebind contracts after reconnect")
		}
	}
	if err := r.paraconn.Reconnect(); err != nil {
		log.WithError(err).Error("Parachain reconnection failed")
	}
}

// rebindContractsLocked refreshes gateway bindings and chainID. Caller must hold ethMu.
func (r *Relay) rebindContractsLocked(ctx context.Context) error {
	address := common.HexToAddress(r.config.Source.Contracts.Gateway)
	contract, err := contracts.NewGateway(address, r.ethconn.Client())
	if err != nil {
		return fmt.Errorf("rebind gateway contract: %w", err)
	}
	r.gatewayContract = contract
	if r.config.V1Enabled() {
		v1Contract, err := contractsv1.NewGateway(address, r.ethconn.Client())
		if err != nil {
			return fmt.Errorf("rebind V1 gateway: %w", err)
		}
		r.gatewayContractV1 = v1Contract
	}

	chainID, err := r.ethconn.Client().NetworkID(ctx)
	if err != nil {
		return fmt.Errorf("refresh chainID: %w", err)
	}
	r.chainID = chainID
	return nil
}

// ErrDuplicateMessage is returned when a message has already been processed
func (r *Relay) filterSkippedNonces(nonces []uint64) []uint64 {
	live := nonces[:0]
	for _, n := range nonces {
		if r.deadNonces.IsDead(n) {
			log.WithField("nonce", n).Debug("skipping dead-lettered nonce")
			continue
		}
		if r.isIgnoredNonce(n) {
			log.WithField("nonce", n).Debug("skipping configured ignored nonce")
			continue
		}
		live = append(live, n)
	}
	return live
}

func (r *Relay) fetchUnprocessedParachainNonces(latest uint64) ([]uint64, error) {
	unprocessedNonces := []uint64{}
	latestBucket := latest / 128

	for b := uint64(0); b <= latestBucket; b++ {
		encodedBucket, err := types.EncodeToBytes(types.NewU64(b))
		bucketKey, _ := types.CreateStorageKey(
			r.paraconn.Metadata(),
			"EthereumInboundQueueV2",
			"NonceBitmap",
			encodedBucket,
			nil,
		)

		var value types.U128
		ok, err := r.paraconn.API().RPC.State.GetStorageLatest(bucketKey, &value)
		if err != nil {
			return nil, fmt.Errorf("failed to read bucket %d: %w", b, err)
		}

		// "Missing" means the chain doesn't store it => it's 0
		if !ok {
			value = types.NewU128(*big.NewInt(0))
		}

		// Now parse bits from value...
		bucketNonces := extractUnprocessedNonces(value, latest, b)
		unprocessedNonces = append(unprocessedNonces, bucketNonces...)
	}

	log.WithFields(log.Fields{
		"nonces": unprocessedNonces,
	}).Debug("nonces to be processed")
	return unprocessedNonces, nil
}

func (r *Relay) isParachainNonceSet(index uint64) (bool, error) {
	log.WithFields(log.Fields{
		"index": index,
	}).Debug("is parachain nonce set")
	// Calculate the bucket and bit position
	bucket := index / 128
	bitPosition := index % 128

	encodedBucket, err := types.EncodeToBytes(types.NewU64(bucket))
	bucketKey, err := types.CreateStorageKey(r.paraconn.Metadata(), "EthereumInboundQueueV2", "NonceBitmap", encodedBucket)
	if err != nil {
		return false, fmt.Errorf("create storage key for EthereumInboundQueueV2.NonceBitmap: %w", err)
	}

	var bucketValue types.U128
	ok, err := r.paraconn.API().RPC.State.GetStorageLatest(bucketKey, &bucketValue)

	if err != nil {
		return false, fmt.Errorf("fetch storage EthereumInboundQueueV2.NonceBitmap keys: %w", err)
	}
	if !ok {
		return false, fmt.Errorf("bucket does not exist: %w", err)
	}

	return checkBitState(bucketValue, bitPosition), nil
}

func checkBitState(bucketValue types.U128, bitPosition uint64) bool {
	log.WithFields(log.Fields{
		"bucketValue": bucketValue,
		"bitPosition": bitPosition,
	}).Debug("checking bit state")
	mask := new(big.Int).Lsh(big.NewInt(1), uint(bitPosition)) // Create mask for the bit position
	result := new(big.Int).And(bucketValue.Int, mask).Cmp(big.NewInt(0)) != 0
	log.WithFields(log.Fields{
		"result":      result,
		"bitPosition": bitPosition,
	}).Debug("check bit state result")
	return result
}

func extractUnprocessedNonces(bitmap types.U128, latest uint64, bucketIndex uint64) []uint64 {
	var unprocessed []uint64
	// Each bucket covers 128 nonces
	baseNonce := bucketIndex * 128

	for i := 0; i < 128; i++ {
		nonce := baseNonce + uint64(i)
		// Ignore nonce 0 since valid nonces start at 1
		if nonce < 1 {
			continue
		}
		// If we've passed the latest nonce to consider, stop checking further bits.
		if nonce > latest {
			break
		}
		// Check if bit `i` is unset (meaning unprocessed).
		mask := new(big.Int).Lsh(big.NewInt(1), uint(i))
		if new(big.Int).And(bitmap.Int, mask).Cmp(big.NewInt(0)) == 0 {
			unprocessed = append(unprocessed, nonce)
		}
	}

	return unprocessed
}

func (r *Relay) fetchEthereumNonceAtBlock(ctx context.Context, blockNumber uint64) (uint64, error) {
	bn := new(big.Int).SetUint64(blockNumber)
	return r.fetchEthereumNonceWithBlockNumber(ctx, bn)
}

func (r *Relay) fetchEthereumNonceWithBlockNumber(ctx context.Context, blockNumber *big.Int) (uint64, error) {
	opts := bind.CallOpts{
		Context:     ctx,
		BlockNumber: blockNumber,
	}
	ethOutboundNonce, err := r.gatewayContract.V2OutboundNonce(&opts)
	if err != nil {
		return 0, fmt.Errorf("fetch Gateway.OutboundNonce: %w", err)
	}

	return ethOutboundNonce, nil
}

// findEvents returns the OutboundMessageAccepted events for nonces, reading only
// the block that emitted each one. Walking back from the tip in getLogs pages
// costs one request per page, which rate-limited RPCs reject for old nonces.
func (r *Relay) findEvents(
	ctx context.Context,
	latestFinalizedBlockNumber uint64,
	nonces []uint64,
) ([]*contracts.GatewayOutboundMessageAccepted, error) {
	blocks, err := r.resolveNonceBlocks(ctx, latestFinalizedBlockNumber, nonces, r.v2NonceBlocks, r.fetchEthereumNonceAtBlock)
	if err != nil {
		return nil, fmt.Errorf("resolve nonce blocks: %w", err)
	}

	var allEvents []*contracts.GatewayOutboundMessageAccepted
	found := make(map[uint64]struct{}, len(blocks))
	for _, block := range distinctBlocks(blocks) {
		end := block
		iter, err := r.gatewayContract.FilterOutboundMessageAccepted(&bind.FilterOpts{Start: block, End: &end, Context: ctx})
		if err != nil {
			return nil, fmt.Errorf("filter events: %w", err)
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
			return nil, fmt.Errorf("filter events: %w", err)
		}
	}

	sort.SliceStable(allEvents, func(i, j int) bool {
		return allEvents[i].Nonce < allEvents[j].Nonce
	})
	r.forgetMissingNonces(r.v2NonceBlocks, blocks, found)

	return allEvents, nil
}

func (r *Relay) makeInboundMessage(
	ctx context.Context,
	headerCache *ethereum.HeaderCache,
	event *contracts.GatewayOutboundMessageAccepted,
) (*parachain.MessageV3, error) {
	receiptTrie, err := headerCache.GetReceiptTrie(ctx, event.Raw.BlockHash)
	if err != nil {
		log.WithFields(log.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to get receipt trie for event")
		return nil, err
	}

	msg, err := ethereum.MakeMessageFromEvent(&event.Raw, receiptTrie)
	if err != nil {
		log.WithFields(log.Fields{
			"address":     event.Raw.Address.Hex(),
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to generate message from ethereum event")
		return nil, err
	}

	log.WithFields(log.Fields{
		"blockHash":   event.Raw.BlockHash.Hex(),
		"blockNumber": event.Raw.BlockNumber,
		"txHash":      event.Raw.TxHash.Hex(),
	}).Info("found message")

	return msg, nil
}

// waitAndSendWithRetry wraps waitAndSend with retry logic for transient errors.
// On a permanent (non-transient) error other than ErrBeaconHeaderNotFinalized,
// the nonce is recorded in the dead-nonce store before the error is returned,
// so the same nonce will be skipped on subsequent runs (including after a
// process restart).
func (r *Relay) fetchUnprocessedNonceSet(latest uint64) (map[uint64]struct{}, error) {
	nonces, err := r.fetchUnprocessedParachainNonces(latest)
	if err != nil {
		return nil, err
	}
	set := make(map[uint64]struct{}, len(nonces))
	for _, n := range nonces {
		set[n] = struct{}{}
	}
	return set, nil
}

func (r *Relay) getTransactionSender(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted) (string, error) {
	tx, _, err := r.ethconn.Client().TransactionByHash(ctx, ev.Raw.TxHash)
	if err != nil {
		return "", err
	}

	sender, err := ethtypes.Sender(ethtypes.LatestSignerForChainID(r.chainID), tx)
	if err != nil {
		return "", fmt.Errorf("retrieve message sender: %w", err)
	}

	log.WithFields(log.Fields{
		"sender": sender,
	}).Debug("extracted sender from transaction")

	return sender.Hex(), nil
}

func (r *Relay) handleCandidateError(nonce uint64, err error) error {
	if err == nil {
		return nil
	}
	if error_tracking.IsTransientError(err) {
		return err
	}
	log.WithError(err).WithField("nonce", nonce).Warn("[instant-eth] Non-transient candidate error, marking nonce dead")
	if markErr := r.deadNonces.MarkDead(nonce, err.Error()); markErr != nil {
		log.WithError(markErr).WithField("nonce", nonce).Error("[instant-eth] failed to persist dead-nonce record")
	}
	return nil
}

type batchCandidate struct {
	ev               *contracts.GatewayOutboundMessageAccepted
	inboundMsg       *parachain.MessageV3
	parentBeaconRoot common.Hash
	source           string
	headerSlot       uint64
	needsConsensus   bool
	proof            scale.ProofPayload
	quote            *v2Economics
}

// buildCandidate resolves the inbound message, transaction sender, and beacon
// header for an event. The caller pre-filters events against the unprocessed-nonce
// set fetched once at the top of the scan, and the authoritative processed re-check
// happens once just before submission, so no per-candidate bitmap scan is done here.
// Returns a non-nil (unwrapped) error for the caller to classify via
// handleCandidateError.
func (r *Relay) buildCandidate(ctx context.Context, ev *contracts.GatewayOutboundMessageAccepted, onchainFinalizedSlot uint64) (*batchCandidate, error) {
	inboundMsg, err := r.makeInboundMessage(ctx, r.headerCache, ev)
	if err != nil {
		return nil, err
	}

	source, err := r.getTransactionSender(ctx, ev)
	if err != nil {
		return nil, err
	}

	beaconRoot, err := r.beaconRootForEvent(ctx, ev)
	if err != nil {
		return nil, err
	}

	header, err := r.beaconAPI.GetHeaderByBlockRoot(beaconRoot)
	if err != nil {
		return nil, err
	}

	return &batchCandidate{
		ev:               ev,
		inboundMsg:       inboundMsg,
		parentBeaconRoot: beaconRoot,
		source:           source,
		headerSlot:       header.Slot,
		needsConsensus:   header.Slot > onchainFinalizedSlot,
	}, nil
}

// beaconRootForEvent returns the beacon block root that commits to the execution block
// containing the gateway event.
//
// Pre-Gloas uses ParentBeaconRoot of the next execution block (EIP-4788). From Gloas the
// proven leaf is parent_block_hash on a later successor beacon block, so we resolve that
// successor from the message block's slotNumber instead.
func (r *Relay) beaconRootForEvent(ctx context.Context, event *contracts.GatewayOutboundMessageAccepted) (common.Hash, error) {
	payloadSlot, hasSlot, err := execution.FetchHeaderSlotNumber(ctx, r.ethconn.Client(), event.Raw.BlockHash)
	if err != nil {
		return common.Hash{}, fmt.Errorf("fetch message block slotNumber: %w", err)
	}
	if hasSlot {
		return r.beaconHeader.BeaconRootCommittingToExecutionBlock(payloadSlot, event.Raw.BlockHash)
	}

	nextBlockNumber := new(big.Int).SetUint64(event.Raw.BlockNumber + 1)
	blockHeader, err := r.ethconn.Client().HeaderByNumber(ctx, nextBlockNumber)
	if err != nil {
		return common.Hash{}, fmt.Errorf("get next block header: %w", err)
	}
	if blockHeader.ParentBeaconRoot == nil {
		return common.Hash{}, fmt.Errorf("block %d has no parentBeaconBlockRoot", nextBlockNumber.Uint64())
	}
	return *blockHeader.ParentBeaconRoot, nil
}

// executionBlockNumberFromBeaconBlock resolves the execution block number the finalized
// checkpoint's beacon block commits to.
//
// Pre-Gloas: execution payload lives on the beacon block body. Gloas (EIP-7732): the body
// carries a signed execution payload bid instead, and Snowbridge proves parent_block_hash
// from that bid, so its EL block number is resolved via the execution node.
func (r *Relay) executionBlockNumberFromBeaconBlock(ctx context.Context, beaconBlock *api.BeaconBlockResponse, blockRoot common.Hash) (uint64, error) {
	if bn := beaconBlock.Data.Message.Body.ExecutionPayload.BlockNumber; bn != "" {
		blockNumber, err := strconv.ParseUint(bn, 10, 64)
		if err != nil {
			return 0, fmt.Errorf("parse execution payload block number: %w", err)
		}
		return blockNumber, nil
	}

	bid := beaconBlock.Data.Message.Body.SignedExecutionPayloadBid
	if bid == nil || bid.Message.ParentBlockHash == "" {
		return 0, fmt.Errorf("beacon block %s has neither execution_payload.block_number nor signed_execution_payload_bid.parent_block_hash", blockRoot.Hex())
	}
	header, err := r.ethconn.Client().HeaderByHash(ctx, common.HexToHash(bid.Message.ParentBlockHash))
	if err != nil {
		return 0, fmt.Errorf("resolve parent_block_hash %s to block number: %w", bid.Message.ParentBlockHash, err)
	}
	return header.Number.Uint64(), nil
}

func (r *Relay) doScanAndBatchRelay(ctx context.Context, checkpoint api.FinalizedCheckpoint, kind wakeKind) error {
	// Hold ethMu for the whole scan/submit so reconnectAndRebind cannot swap
	// gateway/chainID (or reconnect shared RPCs) mid-flight.
	r.ethMu.RLock()
	defer r.ethMu.RUnlock()

	beaconBlock, err := r.beaconAPI.GetBeaconBlock(checkpoint.FinalizedBlockRoot)
	if err != nil {
		return fmt.Errorf("get beacon block for root %s: %w", checkpoint.FinalizedBlockRoot.Hex(), err)
	}

	// The checkpoint's beacon block gives both the finalized slot and the execution
	// block number, so no separate finalized-update fetch is needed just for the slot.
	finalizedSlot, err := util.ToUint64(beaconBlock.Data.Message.Slot)
	if err != nil {
		return fmt.Errorf("parse finalized beacon block slot: %w", err)
	}

	r.mu.Lock()
	r.latestFinalizedSlot = finalizedSlot
	r.mu.Unlock()

	log.WithFields(log.Fields{
		"slot": finalizedSlot,
		"wake": kind.String(),
	}).Info("[instant-eth] Scanning for batch relay task")

	executionBlockNumber, err := r.executionBlockNumberFromBeaconBlock(ctx, &beaconBlock, checkpoint.FinalizedBlockRoot)
	if err != nil {
		return err
	}

	ethNonce, err := r.fetchEthereumNonceAtBlock(ctx, executionBlockNumber)
	if err != nil {
		return fmt.Errorf("fetch Gateway.OutboundNonce at finalized block %d: %w", executionBlockNumber, err)
	}

	paraNonces, err := r.fetchUnprocessedParachainNonces(ethNonce)
	if err != nil {
		return err
	}

	paraNonces = r.filterSkippedNonces(paraNonces)
	r.pruneDeferred(paraNonces)
	now := time.Now()

	log.WithFields(log.Fields{
		"paraNonces":           paraNonces,
		"ethNonce":             ethNonce,
		"executionBlockNumber": executionBlockNumber,
	}).Info("[instant-eth] Polled nonces for batch relay")

	// Best-block view matches BH LC watch and peer delivery; finalized-head lag would
	// falsely keep needsConsensus set after a competing LC submit.
	lastFinalizedHeaderState, err := r.writer.GetLastFinalizedHeaderStateAtBestBlock()
	if err != nil {
		return fmt.Errorf("fetch last finalized header state: %w", err)
	}

	// Cap the number of built candidates to fit exactly in a single force_batch
	// extrinsic, and stop building once the cap is reached so we don't pay the
	// per-candidate fetch/proof/estimate cost for messages that won't be submitted
	// this round. Events are nonce-ordered, so this keeps the oldest deliverable ones.
	maxMessages := int(r.writer.MaxWatchedExtrinsics()) - 1
	if maxMessages <= 0 {
		maxMessages = 1
	}

	var candidates []batchCandidate
	includeBeacon := false

	if len(paraNonces) > 0 {
		events, err := r.findEvents(ctx, executionBlockNumber, paraNonces)
		if err != nil {
			return fmt.Errorf("find events: %w", err)
		}

		nonceSet := make(map[uint64]struct{}, len(paraNonces))
		for _, nonce := range paraNonces {
			nonceSet[nonce] = struct{}{}
		}

		for _, ev := range events {
			if len(candidates) >= maxMessages {
				log.WithField("cap", maxMessages).Info("[instant-eth] Reached force_batch candidate cap; skipping remaining messages this round")
				break
			}
			if _, ok := nonceSet[ev.Nonce]; !ok {
				continue
			}
			if !r.passesStaticFeeFloor(ev) && !v2PastMaxDelay(ev, executionBlockNumber) {
				log.WithField("nonce", ev.Nonce).Debug("[instant-eth] RelayerFee+serviceFee below static floor; ignoring")
				continue
			}
			if r.isDeferred(ev.Nonce, now) && !v2PastMaxDelay(ev, executionBlockNumber) {
				continue
			}

			c, err := r.buildCandidate(ctx, ev, lastFinalizedHeaderState.BeaconSlot)
			if err != nil {
				if errHandled := r.handleCandidateError(ev.Nonce, err); errHandled != nil {
					return errHandled
				}
				continue
			}

			if c.needsConsensus {
				includeBeacon = true
			}
			candidates = append(candidates, *c)
		}
	}

	if len(candidates) == 0 {
		log.Info("[instant-eth] No V2 candidates; will still try V1 piggyback")
	}

	var sharedFinalizedPayload *scale.Update
	var checkpointProof cache.Proof
	// BH LC wakes never co-submit beacon: on-chain LC already moved. For SSE/startup,
	// only submit when Lodestar's LC finality_update is strictly ahead of on-chain.
	// Dedicated beacon-1 / peers land lagging checkpoints; BH LC watch then delivers
	// message-only.
	if includeBeacon && kind.allowBeaconSubmit() && len(candidates) > 0 {
		// Prove against an update that covers every needsConsensus candidate. Prefer the
		// scan checkpoint slot so we land the same epoch peers target.
		requiredSlot := finalizedSlot
		for i := range candidates {
			if candidates[i].needsConsensus && candidates[i].headerSlot > requiredSlot {
				requiredSlot = candidates[i].headerSlot
			}
		}

		finalizedUpdate, updateSlot, err := r.waitForFinalityUpdateAhead(ctx, lastFinalizedHeaderState.BeaconSlot, requiredSlot)
		if err != nil {
			return fmt.Errorf("wait for LC finality_update: %w", err)
		}
		onchainSlot := lastFinalizedHeaderState.BeaconSlot

		if !finalityUpdateCoversRequired(updateSlot, onchainSlot, requiredSlot) {
			log.WithFields(log.Fields{
				"updateSlot":   updateSlot,
				"onchainSlot":  onchainSlot,
				"requiredSlot": requiredSlot,
				"scanSlot":     finalizedSlot,
				"wake":         kind.String(),
			}).Info("[instant-eth] LC finality_update not ready after short-poll; skipping beacon update this round")
			includeBeacon = false
		} else {
			sharedFinalizedPayload = &finalizedUpdate
			checkpointProof = cache.Proof{
				FinalizedBlockRoot: finalizedUpdate.FinalizedHeaderBlockRoot,
				BlockRootsTree:     finalizedUpdate.BlockRootsTree,
				Slot:               updateSlot,
			}
		}
	} else if includeBeacon {
		log.WithFields(log.Fields{
			"wake":        kind.String(),
			"onchainSlot": lastFinalizedHeaderState.BeaconSlot,
		}).Info("[instant-eth] Message-only wake; skipping beacon update attempt")
		includeBeacon = false
	}

	// Generate each candidate's execution proof. Candidates newer than the on-chain
	// finalized header (needsConsensus) are anchored to the shared finalized update
	// being submitted in this batch; the rest are proven against the existing on-chain
	// header. When no beacon update is included, needsConsensus candidates cannot be
	// proven and are skipped.
	for i := range candidates {
		c := &candidates[i]

		var headerUpdate scale.HeaderUpdatePayload
		var finalizedPayload *scale.Update
		var err error

		if c.needsConsensus {
			if !includeBeacon {
				log.WithFields(log.Fields{
					"nonce":         c.ev.Nonce,
					"candidateSlot": c.headerSlot,
					"onchainSlot":   lastFinalizedHeaderState.BeaconSlot,
				}).Info("[instant-eth] candidate requires finality update but none is being included; skipping")
				continue
			}
			if c.headerSlot > checkpointProof.Slot {
				log.WithFields(log.Fields{
					"nonce":          c.ev.Nonce,
					"candidateSlot":  c.headerSlot,
					"checkpointSlot": checkpointProof.Slot,
				}).Warn("[instant-eth] candidate slot not finalized under latest update yet")
				continue
			}
			headerUpdate, err = r.beaconHeader.GetHeaderUpdate(c.parentBeaconRoot, &checkpointProof)
			finalizedPayload = sharedFinalizedPayload
		} else {
			// Message already covered by on-chain LC: prove the exact parent beacon root
			// against that finalized checkpoint (not a slot-search substitute).
			headerUpdate, err = r.beaconHeader.GetHeaderUpdateAgainstOnchainFinalized(c.parentBeaconRoot)
		}

		if err != nil {
			if errHandled := r.handleCandidateError(c.ev.Nonce, err); errHandled != nil {
				return errHandled
			}
			continue
		}
		c.proof = scale.ProofPayload{
			HeaderPayload:    headerUpdate,
			FinalizedPayload: finalizedPayload,
		}
	}

	var validCandidates []batchCandidate
	for _, c := range candidates {
		if c.proof.HeaderPayload.Header.Slot > 0 {
			validCandidates = append(validCandidates, c)
		}
	}

	// Attach the execution proof to each message.
	for i := range validCandidates {
		validCandidates[i].inboundMsg.Proof.ExecutionProof = validCandidates[i].proof.HeaderPayload
	}

	// Final filtering: OFAC + already-processed nonces; profitability via checkV2Profitable.
	var finalEvs []*contracts.GatewayOutboundMessageAccepted
	var finalMessages []*parachain.MessageV3
	var finalProofs []scale.ProofPayload
	var finalSources []string
	var finalQuotes []*v2Economics

	if len(validCandidates) > 0 {
		// Single authoritative bitmap read for the pre-submit processed re-check.
		unprocessedBeforeSubmit, err := r.fetchUnprocessedNonceSet(ethNonce)
		if err != nil {
			return fmt.Errorf("fetch unprocessed nonce set for final check: %w", err)
		}

		for i := range validCandidates {
			c := &validCandidates[i]

			if _, unprocessed := unprocessedBeforeSubmit[c.ev.Nonce]; !unprocessed {
				continue // already processed by an earlier submission or a competing relayer
			}

			c.inboundMsg.Proof.ExecutionProof = c.proof.HeaderPayload
			beneficiaries, quote, profitable, err := r.checkV2Profitable(ctx, c.ev, c.inboundMsg, c.source, executionBlockNumber)
			if err != nil {
				if errHandled := r.handleCandidateError(c.ev.Nonce, err); errHandled != nil {
					return errHandled
				}
				continue
			}
			if !profitable {
				r.deferNonce(c.ev.Nonce, now.Add(dryRunRetry))
				continue
			}
			if !quote.coversOwnDelivery() {
				log.WithFields(log.Fields{
					"nonce":       c.ev.Nonce,
					"rewardWei":   quote.reward.String(),
					"deliveryWei": quote.delivery.String(),
				}).Info("[instant-eth] V2 reward below its own delivery fee; deferring")
				r.deferNonce(c.ev.Nonce, now.Add(underpaidRetry))
				continue
			}
			c.quote = quote

			banned, err := r.ofac.IsBanned(c.source, beneficiaries)
			if err != nil {
				log.WithError(err).WithField("nonce", c.ev.Nonce).Warn("[instant-eth] OFAC check failed; skipping nonce this round")
				continue
			}
			if banned {
				log.WithField("nonce", c.ev.Nonce).Warn("[instant-eth] banned sender/beneficiary; dropping message from batch")
				continue
			}

			finalEvs = append(finalEvs, c.ev)
			finalMessages = append(finalMessages, c.inboundMsg)
			finalProofs = append(finalProofs, c.proof)
			finalSources = append(finalSources, c.source)
			finalQuotes = append(finalQuotes, c.quote)
		}
	}

	// V1 always piggybacks on tip (no fee gate), proven against on-chain LC or this batch's update.
	// A collect failure must not exit the worker: Header.Sync shares this process, and a
	// non-transient error here cancels beacon catch-up too.
	v1Cands, err := r.collectV1Piggybacks(ctx, executionBlockNumber, lastFinalizedHeaderState.BeaconSlot, sharedFinalizedPayload)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] V1 collect failed; submitting V2 batch without V1")
		v1Cands = nil
	}

	if len(finalEvs) == 0 && len(v1Cands) == 0 {
		log.Info("[instant-eth] No deliverable V1/V2 messages; skipping")
		return nil
	}

	log.WithFields(log.Fields{
		"messageCount":  len(finalEvs),
		"v1Count":       len(v1Cands),
		"includeBeacon": includeBeacon,
	}).Info("[instant-eth] Submitting batch")

	// No pending-force_batch deferral: v3 competes with external relayers and submits
	// whenever it has deliverable messages. Losing a race only wastes gas on
	// already-processed items (non-atomic force_batch); tip and bitmap re-check bound cost.
	// Writer mutex serializes with catch-up Header.Sync submits.
	var calls []batchCall

	if includeBeacon && sharedFinalizedPayload != nil {
		calls = append(calls, batchCall{
			name:    "EthereumBeaconClient.submit",
			payload: sharedFinalizedPayload.Payload,
		})
		log.WithFields(log.Fields{
			"slot": sharedFinalizedPayload.Payload.FinalizedHeader.Slot,
		}).Info("[instant-eth] Including EthereumBeaconClient.submit in force_batch")
	}

	for i, ev := range finalEvs {
		inboundMsg := finalMessages[i]
		inboundMsg.Proof.ExecutionProof = finalProofs[i].HeaderPayload

		submitPayload, err := r.paraconn.GenerateSubmitMessagePayload(inboundMsg)
		if err != nil {
			return fmt.Errorf("generate submit message payload for nonce %d: %w", ev.Nonce, err)
		}

		calls = append(calls, batchCall{
			name:    "EthereumInboundQueueV2.submit",
			payload: submitPayload,
			v2:      finalQuotes[i],
		})
		log.WithField("nonce", ev.Nonce).Info("[instant-eth] Including EthereumInboundQueueV2.submit in force_batch")
	}

	v1Extrinsics, v1Payloads, err := r.appendV1ToBatch(nil, nil, v1Cands)
	if err != nil {
		return err
	}
	for i := range v1Extrinsics {
		calls = append(calls, batchCall{name: v1Extrinsics[i], payload: v1Payloads[i]})
	}

	calls, err = r.selectProfitableBatch(ctx, calls)
	if err != nil {
		if error_tracking.IsTransientError(err) {
			return err
		}
		log.WithError(err).Warn("[instant-eth] batch dry-run failed; skipping this round")
		return nil
	}
	// A beacon update with no surviving message is catch-up's job.
	if !batchHasMessage(calls) {
		log.Info("[instant-eth] no messages left after the batch gate; skipping")
		return nil
	}

	submittedV2 := make(map[uint64]struct{})
	for _, c := range calls {
		if c.v2 != nil {
			submittedV2[c.v2.nonce] = struct{}{}
		}
	}

	extrinsics, payloads := callsToForceBatch(calls)
	err = r.writer.ForceBatch(ctx, extrinsics, payloads)
	if err != nil {
		return fmt.Errorf("submit force_batch to parachain failed: %w", err)
	}

	log.Info("[instant-eth] force_batch extrinsic submitted successfully; verifying outcomes...")

	// Single bitmap read to reconcile per-nonce outcomes instead of scanning per nonce.
	unprocessedAfterSubmit, err := r.fetchUnprocessedNonceSet(ethNonce)
	if err != nil {
		log.WithError(err).Warn("[instant-eth] Failed to fetch nonce bitmap for reconciliation; skipping per-nonce verification")
		return nil
	}

	var succeededCount int
	var failedNonces []uint64
	for _, ev := range finalEvs {
		if _, submitted := submittedV2[ev.Nonce]; !submitted {
			continue
		}
		if _, stillUnprocessed := unprocessedAfterSubmit[ev.Nonce]; stillUnprocessed {
			failedNonces = append(failedNonces, ev.Nonce)
			log.WithField("nonce", ev.Nonce).Warn("[instant-eth] Message submission failed (inner item failed)")
		} else {
			succeededCount++
			log.WithField("nonce", ev.Nonce).Info("[instant-eth] Message successfully processed")
		}
	}

	log.WithFields(log.Fields{
		"succeeded":  succeededCount,
		"failed":     len(failedNonces),
		"failedList": failedNonces,
	}).Info("[instant-eth] force_batch reconciliation complete")

	return nil
}
