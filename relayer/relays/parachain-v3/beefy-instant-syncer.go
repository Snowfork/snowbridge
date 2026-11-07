package parachainV3

import (
	"context"
	"fmt"
	"math/big"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	geth "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	contractsv1 "github.com/snowfork/snowbridge/relayer/contracts/v1"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
	"golang.org/x/sync/errgroup"
)

type DryRunMetrics struct {
	EstimatedGas       uint64
	GasPriceWei        *big.Int
	EstimatedFeeWei    *big.Int
	ConservativeFeeWei *big.Int
	// Gas limit from strict dry-run after bump; apply to broadcast tx opts.
	EffectiveGasLimit uint64
}

// Outer gas needed so inner frames still pass Gateway gasleft()*63/64 after `hops` EIP-150 retentions.
func gasWithForwardingReserve(gas uint64, hops int) uint64 {
	for i := 0; i < hops; i++ {
		gas = (gas*64 + 62) / 63
	}
	return gas
}

// Fee pricing uses expected dispatch burn, not MaxDispatchGas ceiling.
func expectedCommandDispatchGas(command *CommandWrapper) uint64 {
	switch command.Kind {
	case 2: // ERC20 unlock/transfer
		return BaseUnlockGas
	case 4: // PNA mint
		return BaseMintGas
	default:
		return uint64(command.MaxDispatchGas)
	}
}

func minGasForV2SubmitProof(proof *MessageProof) uint64 {
	gas := gasWithForwardingReserve(BaseMessageVerificationGas, 1)
	for _, cmd := range proof.Message.OriginalMessage.Commands {
		gas += gasWithForwardingReserve(uint64(cmd.MaxDispatchGas)+gatewayDispatchOverheadBuffer, 3)
	}
	return gas
}

func estimatedGasForV2SubmitProof(proof *MessageProof) uint64 {
	gas := gasWithForwardingReserve(BaseMessageVerificationGas, 1)
	for _, cmd := range proof.Message.OriginalMessage.Commands {
		gas += gasWithForwardingReserve(expectedCommandDispatchGas(&cmd)+gatewayDispatchOverheadBuffer, 3)
	}
	return gas
}

func minGasForV1SubmitProof(proof *V1MessageProof) uint64 {
	gas := gasWithForwardingReserve(BaseMessageVerificationGas, 1)
	gas += gasWithForwardingReserve(proof.Message.MaxDispatchGas+gatewayDispatchOverheadBuffer, 2)
	return gas
}

func multicallGasFloor(minSubmitProofGasSum uint64) uint64 {
	return BaseBeefyFiatShamirGas + multicallBeefyGasHeadroom + multicall3AggregateOverheadBuffer + minSubmitProofGasSum
}

// Raise limit when estimate is low because AllowFailure=true hides inner v2_submit reverts.
func (li *BeefyInstantSyncer) applyMulticallGasFloor(opts *bind.TransactOpts, minSubmitProofGasSum uint64) {
	minTotal := multicallGasFloor(minSubmitProofGasSum)
	if opts.GasLimit == 0 || opts.GasLimit < minTotal {
		if opts.GasLimit > 0 {
			log.WithFields(log.Fields{
				"configuredGasLimit": opts.GasLimit,
				"minimumGasLimit":    minTotal,
			}).Info("Raising Multicall3 tx gas limit so nested v2_submit can forward command gas")
		} else {
			log.WithField("minimumGasLimit", minTotal).Info("Setting Multicall3 tx gas limit from computed minimum (sink gas-limit was unset)")
		}
		opts.GasLimit = minTotal
	}
}

func strictAggregate3Calls(calls []contracts.Multicall3Call3) []contracts.Multicall3Call3 {
	out := make([]contracts.Multicall3Call3, len(calls))
	for i, c := range calls {
		out[i] = c
		out[i].AllowFailure = false
	}
	return out
}

func applyEffectiveGasLimit(opts *bind.TransactOpts, metrics *DryRunMetrics) {
	if metrics == nil || opts == nil {
		return
	}
	if metrics.EffectiveGasLimit > opts.GasLimit {
		log.WithFields(log.Fields{
			"previousGasLimit":  opts.GasLimit,
			"effectiveGasLimit": metrics.EffectiveGasLimit,
		}).Info("Raising Multicall3 tx gas limit to match successful strict dry-run")
		opts.GasLimit = metrics.EffectiveGasLimit
	}
}

type BeefyInstantSyncer struct {
	config             *Config
	beefyListener      *BeefyListener
	beefyOnDemandRelay *beefy.OnDemandRelay
	ethereumWriter     *EthereumWriter
	multicall3         *contracts.Multicall3
	beefyClientABI     abi.ABI
	serviceFees        serviceFeeLookup
	submitMu sync.Mutex // nonce + broadcast for instant/direct-relay/handover/BEEFY steps
	handoverInFlight atomic.Bool
	mu sync.Mutex // WS instant vs catch-up scan; handover async outside (nonce via submitMu)
}

func NewBeefyInstantSyncer(
	config *Config,
	beefyListener *BeefyListener,
	beefyOnDemandRelay *beefy.OnDemandRelay,
	ethereumWriter *EthereumWriter,
) (*BeefyInstantSyncer, error) {
	if err := validateProfitabilityConfig(); err != nil {
		return nil, fmt.Errorf("invalid profitability config: %w", err)
	}

	beefyClientABI, err := abi.JSON(strings.NewReader(contracts.BeefyClientABI))
	if err != nil {
		return nil, fmt.Errorf("parse beefy client ABI: %w", err)
	}

	li := &BeefyInstantSyncer{
		config:             config,
		beefyListener:      beefyListener,
		beefyOnDemandRelay: beefyOnDemandRelay,
		ethereumWriter:     ethereumWriter,
		beefyClientABI:     beefyClientABI,
		serviceFees:        newServiceFeeLookupFromEnv(),
	}
	if beefyOnDemandRelay != nil {
		beefyOnDemandRelay.SetSubmitMutex(&li.submitMu)
	}
	return li, nil
}

func (li *BeefyInstantSyncer) Start(ctx context.Context, eg *errgroup.Group) error {
	err := li.beefyListener.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize beefy listener: %w", err)
	}
	err = li.ethereumWriter.initialize()
	if err != nil {
		return fmt.Errorf("initialize ethereum writer: %w", err)
	}
	// Bind Multicall3 only after ConnectWithHeartBeat or Transact panics on nil client.
	client := li.ethereumWriter.conn.Client()
	if client == nil {
		return fmt.Errorf("ethereum writer client is nil; connect sink ethereum before starting instant syncer")
	}
	li.multicall3, err = contracts.NewMulticall3(common.HexToAddress(li.config.Sink.Contracts.Multicall3), client)
	if err != nil {
		return fmt.Errorf("create multicall3: %w", err)
	}
	err = li.beefyOnDemandRelay.InitializeOnDemandSync(ctx, eg)
	if err != nil {
		return fmt.Errorf("initialize on-demand relay: %w", err)
	}

	log.Info("Starting parachain-v3 relayer (WS instant + catch-up)")

	var latestReceivedBlock uint64
	var mu sync.Mutex
	trigger := make(chan struct{}, 1)

	eg.Go(func() error {
		if ctx.Err() != nil {
			return nil
		}
		finalizedBeefyBlockHash, err := li.beefyListener.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
		if err != nil {
			log.WithError(err).Warn("Failed to fetch latest beefy finalized head for initial sync")
			return nil
		}
		finalizedBeefyBlockHeader, err := li.beefyListener.relaychainConn.API().RPC.Chain.GetHeader(finalizedBeefyBlockHash)
		if err != nil {
			log.WithError(err).Warn("Failed to fetch block header for initial sync")
			return nil
		}
		log.WithField("blockNumber", finalizedBeefyBlockHeader.Number).Info("[instant-v2] Triggering initial startup sync")
		err = li.doScanAndInstantRelayV2(ctx, uint64(finalizedBeefyBlockHeader.Number))
		if err != nil {
			if error_tracking.IsTransientError(err) {
				li.handleTransientEthError(ctx, err, "Transient error during initial startup sync")
			} else {
				return fmt.Errorf("fatal error during initial startup sync: %w", err)
			}
		}
		return nil
	})

	eg.Go(func() error {
		for {
			select {
			case <-ctx.Done():
				return nil
			case <-trigger:
			}

			mu.Lock()
			blockNum := latestReceivedBlock
			mu.Unlock()

			if blockNum == 0 {
				continue
			}

			log.WithField("blockNumber", blockNum).Info("[instant-v2] Worker processing WebSocket BEEFY justification event")
			err := li.doScanAndInstantRelayV2(ctx, blockNum)
			if err != nil {
				if error_tracking.IsTransientError(err) {
					li.handleTransientEthError(ctx, err, "Transient error in instant subscription relay path, worker will retry on next event")
				} else {
					return fmt.Errorf("fatal error in instant subscription relay path: %w", err)
				}
			}
		}
	})

	eg.Go(func() error {
		for {
			if ctx.Err() != nil {
				return nil
			}

			ctxSub, cancelSub := context.WithTimeout(ctx, 30*time.Second)
			ch := make(chan VersionedSignedCommitment)
			cl := li.beefyListener.relaychainConn.API().Client
			sub, err := cl.Subscribe(ctxSub, "beefy", "subscribeJustifications", "unsubscribeJustifications",
				"justifications", ch)
			cancelSub()
			if err != nil {
				log.WithError(err).Warn("Failed to subscribe to BEEFY justifications, retrying in 5 seconds")
				select {
				case <-ctx.Done():
					return nil
				case <-time.After(5 * time.Second):
					continue
				}
			}

			log.Info("Successfully subscribed to BEEFY justifications")

			err = func() error {
				defer sub.Unsubscribe()
				for {
					select {
					case <-ctx.Done():
						return ctx.Err()
					case err := <-sub.Err():
						if err != nil {
							log.WithError(err).Warn("BEEFY justification subscription error, reconnecting")
							return nil
						}
					case commitment, ok := <-ch:
						if !ok {
							log.Warn("BEEFY justification subscription channel closed, reconnecting")
							return nil
						}

						if len(commitment.Signatures) == 0 {
							log.Debug("BEEFY commitment has no signatures, skipping...")
							continue
						}

						blockNum := uint64(commitment.Commitment.BlockNumber)
						mu.Lock()
						if blockNum > latestReceivedBlock {
							latestReceivedBlock = blockNum
						}
						mu.Unlock()

						select {
						case trigger <- struct{}{}:
						default:
						}
					}
				}
			}()
			if err != nil {
				return nil
			}
		}
	})

	catchUpInterval := time.Duration(FastPathCatchUpIntervalSecs) * time.Second
	if catchUpInterval <= 0 {
		catchUpInterval = 120 * time.Second
	}
	catchUpTicker := time.NewTicker(catchUpInterval)
	eg.Go(func() error {
		defer catchUpTicker.Stop()
		for {
			select {
			case <-ctx.Done():
				return nil
			case <-catchUpTicker.C:
			}

			if ctx.Err() != nil {
				return nil
			}

			log.Info("[catch-up] Triggering catch-up tick")
			err := li.doCatchUp(ctx)
			if err != nil {
				if error_tracking.IsTransientError(err) {
					li.handleTransientEthError(ctx, err, "Transient error in catch-up, will retry next interval")
				} else {
					return fmt.Errorf("fatal error in catch-up: %w", err)
				}
			}
		}
	})

	return nil
}

func (li *BeefyInstantSyncer) handleTransientEthError(ctx context.Context, err error, logMsg string) {
	log.WithError(err).Warn(logMsg)
	li.reconnectAndRebindEthereum(ctx)
}

// Rotate read Ethereum RPC on transient errors; write/Flashbots unchanged.
func (li *BeefyInstantSyncer) reconnectAndRebindEthereum(ctx context.Context) {
	rotated := map[*ethereum.Connection]struct{}{}
	rotate := func(name string, conn *ethereum.Connection) {
		if conn == nil {
			return
		}
		if _, ok := rotated[conn]; ok {
			return
		}
		rotated[conn] = struct{}{}
		if recErr := conn.Reconnect(ctx); recErr != nil {
			log.WithError(recErr).WithField("conn", name).Error("Ethereum reconnect after transient error failed")
		}
	}

	if wr := li.ethereumWriter; wr != nil {
		rotate("read", wr.readConn)
		if rebindErr := wr.rebindGateways(); rebindErr != nil {
			log.WithError(rebindErr).Error("Failed to rebind writer gateways after reconnect")
		}
	}
	if li.beefyListener != nil {
		rotate("beefy", li.beefyListener.ethereumConn)
		if rebindErr := li.beefyListener.rebindEthereumContracts(); rebindErr != nil {
			log.WithError(rebindErr).Error("Failed to rebind scanner contracts after reconnect")
		}
	}
	if li.beefyOnDemandRelay != nil {
		li.beefyOnDemandRelay.RotateEthereumEndpoint(ctx)
	}
}

// Decodes beefy_subscribeJustifications VersionedFinalityProof hex.
type VersionedSignedCommitment struct {
	types.SignedCommitment
}

func (v *VersionedSignedCommitment) UnmarshalText(text []byte) error {
	var osc types.OptionalSignedCommitment
	err := types.DecodeFromHexString(string(text), &osc)
	if err != nil {
		return err
	}
	ok, val := osc.Unwrap()
	if !ok {
		return fmt.Errorf("decoding versioned finality proof: got None/empty")
	}
	v.SignedCommitment = val
	return nil
}

func (li *BeefyInstantSyncer) logMulticallReceipt(receipt *gethTypes.Receipt) error {
	for _, ev := range receipt.Logs {
		if len(ev.Topics) == 0 {
			continue
		}

		switch {
		case ev.Topics[0] == li.ethereumWriter.gatewayABI.Events["InboundMessageDispatched"].ID:
			var holder contracts.GatewayInboundMessageDispatched
			if err := li.ethereumWriter.gatewayABI.UnpackIntoInterface(&holder, "InboundMessageDispatched", ev.Data); err != nil {
				return fmt.Errorf("unpack InboundMessageDispatched log: %w", err)
			}
			holder.Nonce = ev.Topics[1].Big().Uint64()
			log.WithFields(log.Fields{
				"nonce":   holder.Nonce,
				"success": holder.Success,
				"version": "v2",
			}).Info("Message dispatched in multicall batch")
		case ev.Topics[0] == li.ethereumWriter.gatewayABIV1.Events["InboundMessageDispatched"].ID:
			var holder contractsv1.GatewayInboundMessageDispatched
			if err := li.ethereumWriter.gatewayABIV1.UnpackIntoInterface(&holder, "InboundMessageDispatched", ev.Data); err != nil {
				return fmt.Errorf("unpack v1 InboundMessageDispatched log: %w", err)
			}
			log.WithFields(log.Fields{
				"nonce":   holder.Nonce,
				"success": holder.Success,
				"version": "v1",
			}).Info("Message dispatched in multicall batch")
		case ev.Topics[0] == li.beefyClientABI.Events["NewMMRRoot"].ID:
			var holder contracts.BeefyClientNewMMRRoot
			if err := li.beefyClientABI.UnpackIntoInterface(&holder, "NewMMRRoot", ev.Data); err != nil {
				return fmt.Errorf("unpack NewMMRRoot log: %w", err)
			}
			log.WithFields(log.Fields{
				"blockNumber": holder.BlockNumber,
				"mmrRoot":     common.Hash(holder.MmrRoot).Hex(),
			}).Info("BEEFY light client updated in multicall batch")
		}
	}

	return nil
}

// Message-only relay at latest on-chain BEEFY block (no consensus prefix).
func (li *BeefyInstantSyncer) tryDirectRelayAnchoredMessages(ctx context.Context, latestBeefyBlockOnEthereum uint64) (bool, error) {
	tasks, err := li.beefyListener.scanner.Scan(ctx, latestBeefyBlockOnEthereum)
	if err != nil {
		return false, fmt.Errorf("scan for sync tasks: %w", err)
	}
	if len(tasks) == 0 {
		return false, nil
	}

	rewardAddress, err := li.rewardAddressBytes()
	if err != nil {
		return false, fmt.Errorf("convert reward address: %w", err)
	}

	relayGasPriceWei, err := li.ethereumWriter.SuggestRelayGasPrice(ctx)
	if err != nil {
		return false, fmt.Errorf("suggest relay gas price: %w", err)
	}

	batch := NewMessageBatchBuilder(li.ethereumWriter.GatewayAddress(), true)
	v2Count, v1Count, err := li.appendMessagesFromScanTasks(
		ctx, batch, tasks, latestBeefyBlockOnEthereum, rewardAddress, relayGasPriceWei, v1FeeFull,
	)
	if err != nil {
		return false, err
	}
	if len(batch.Calls) == 0 {
		return false, nil
	}

	txOpts := li.ethereumWriter.conn.MakeTxOpts(ctx)
	minTotal := multicall3AggregateOverheadBuffer + batch.MinGasSum
	if txOpts.GasLimit == 0 || txOpts.GasLimit < minTotal {
		txOpts.GasLimit = minTotal
	}

	metrics, err := li.simulateAggregate3(ctx, li.ethereumWriter.dryRunClientConn(), batch.Calls, txOpts.GasLimit)
	if err != nil {
		return false, fmt.Errorf("dry-run direct-relay aggregate3 simulation failed: %w", err)
	}
	applyEffectiveGasLimit(txOpts, metrics)

	profitabilityBasisFeeWei := metrics.EstimatedFeeWei
	isProfitable := batch.TotalFeeWei.Cmp(profitabilityBasisFeeWei) >= 0

	fields := log.Fields{
		"blockNumber":                 latestBeefyBlockOnEthereum,
		"estimatedExecutionFeeWei":    metrics.EstimatedFeeWei.String(),
		"conservativeExecutionFeeWei": metrics.ConservativeFeeWei.String(),
		"profitabilityBasisFeeWei":    profitabilityBasisFeeWei.String(),
		"actualCollectedFeeWei":       batch.TotalFeeWei.String(),
		"isProfitable":                isProfitable,
	}
	for k, v := range logBatchComposition(v2Count, v1Count) {
		fields[k] = v
	}
	log.WithFields(fields).Info("[catch-up] Evaluated direct relay profitability for anchored messages")

	if !isProfitable {
		return false, nil
	}

	tx, err := li.submitAggregate3FastPath(ctx, txOpts, batch.Calls)
	if err != nil {
		return false, err
	}
	log.WithFields(log.Fields{
		"txHash":      tx.Hash().Hex(),
		"blockNumber": latestBeefyBlockOnEthereum,
	}).Info("[catch-up] Submitted direct-relay Multicall3 via fast path")
	return true, nil
}

// eth_call dry-run; conn state must match broadcast (fork OK for fiat-shamir, not two-phase Randao writes).
// Strict AllowFailure=false; double gas on revert up to multicallMaxGasLimit. Apply EffectiveGasLimit to broadcast.
func (li *BeefyInstantSyncer) simulateAggregate3(ctx context.Context, conn *ethereum.Connection, calls []contracts.Multicall3Call3, gasLimit uint64) (*DryRunMetrics, error) {
	multicallABI, err := abi.JSON(strings.NewReader(contracts.Multicall3ABI))
	if err != nil {
		return nil, fmt.Errorf("parse multicall3 ABI: %w", err)
	}
	strictCalls := strictAggregate3Calls(calls)
	strictData, err := multicallABI.Pack("aggregate3", strictCalls)
	if err != nil {
		return nil, fmt.Errorf("pack aggregate3 calldata: %w", err)
	}

	to := common.HexToAddress(li.config.Sink.Contracts.Multicall3)
	if gasLimit == 0 {
		gasLimit = multicallMaxGasLimit
	}

	effectiveGas := gasLimit
	for {
		msg := geth.CallMsg{
			From: conn.Keypair().CommonAddress(),
			To:   &to,
			Gas:  effectiveGas,
			Data: strictData,
		}
		_, ethCallErr := conn.Client().CallContract(ctx, msg, nil)
		if ethCallErr == nil {
			break
		}
		if effectiveGas >= multicallMaxGasLimit {
			return nil, fmt.Errorf("strict aggregate3 dry-run failed at gas limit %d: %w", effectiveGas, ethCallErr)
		}
		nextGas := effectiveGas * 2
		if nextGas < effectiveGas || nextGas > multicallMaxGasLimit {
			nextGas = multicallMaxGasLimit
		}
		log.WithFields(log.Fields{
			"previousGasLimit": effectiveGas,
			"nextGasLimit":     nextGas,
			"err":              ethCallErr.Error(),
		}).Warn("Strict aggregate3 dry-run reverted; bumping gas limit and retrying")
		effectiveGas = nextGas
	}

	estimatedGas, err := conn.Client().EstimateGas(ctx, geth.CallMsg{
		From: conn.Keypair().CommonAddress(),
		To:   &to,
		Data: strictData,
	})
	if err != nil {
		return nil, fmt.Errorf("estimate gas for aggregate3: %w", err)
	}

	// Profitability uses live mainnet base fee, not the fork/dry-run conn SuggestGasPrice.
	liveHeader, err := li.ethereumWriter.readConn.Client().HeaderByNumber(ctx, nil)
	if err != nil || liveHeader == nil || liveHeader.BaseFee == nil {
		return nil, fmt.Errorf("fetch live base fee for aggregate3 fee estimate: %w", err)
	}
	gasPriceWei := liveHeader.BaseFee

	feeGas := estimatedGas // expected burn, not padded tx gas limit
	estimatedFeeWei := new(big.Int).Mul(gasPriceWei, new(big.Int).SetUint64(feeGas))
	conservativeFeeWei := new(big.Int).Mul(gasPriceWei, new(big.Int).SetUint64(effectiveGas))

	log.WithFields(log.Fields{
		"estimatedGas":       estimatedGas,
		"feeGas":             feeGas,
		"gasLimit":           effectiveGas,
		"baseFeeWei":         gasPriceWei.String(),
		"estimatedFeeWei":    estimatedFeeWei.String(),
		"conservativeFeeWei": conservativeFeeWei.String(),
	}).Debug("[instant-v2] aggregate3 fee-estimate components (priced at live base fee)")

	return &DryRunMetrics{
		EstimatedGas:       feeGas,
		GasPriceWei:        gasPriceWei,
		EstimatedFeeWei:    estimatedFeeWei,
		ConservativeFeeWei: conservativeFeeWei,
		EffectiveGasLimit:  effectiveGas,
	}, nil
}

func (li *BeefyInstantSyncer) evaluateFastPathProfitability(
	ctx context.Context,
	batch *MessageBatchBuilder,
	gasLimit uint64,
	fiatShamirCalls []contracts.Multicall3Call3,
) (profitable bool, metrics *DryRunMetrics, basisFeeWei *big.Int, pricingMode string, err error) {
	if FastPathDryRun {
		pricingMode = "fork-dry-run"
		metrics, err = li.simulateAggregate3(ctx, li.ethereumWriter.dryRunClientConn(), fiatShamirCalls, gasLimit)
		if err != nil {
			return false, nil, nil, pricingMode, fmt.Errorf("dry-run fiat-shamir aggregate3 simulation failed: %w", err)
		}
		basisFeeWei = fastPathProfitabilityBasisFeeWei(metrics.EstimatedFeeWei)
		profitable = basisFeeWei != nil && batch.TotalFeeWei.Cmp(basisFeeWei) >= 0
		return profitable, metrics, basisFeeWei, pricingMode, nil
	}

	pricingMode = "static-gas"
	metrics, profitable, err = li.evaluateStaticFiatShamirProfitability(
		ctx, batch.FeeGasSum, batch.TotalFeeWei, gasLimit,
	)
	if err != nil {
		return false, nil, nil, pricingMode, err
	}
	basisFeeWei = fastPathProfitabilityBasisFeeWei(metrics.EstimatedFeeWei)
	return profitable, metrics, basisFeeWei, pricingMode, nil
}

// WS hot path: fiat-shamir + ≥1 V2; V1 piggyback fee=0; V1-only deferred to catch-up.
func (li *BeefyInstantSyncer) doScanAndInstantRelayV2(ctx context.Context, beefyBlockNumber uint64) error {
	li.mu.Lock()
	defer li.mu.Unlock()

	consensusResult, err := li.beefyOnDemandRelay.BuildConsensusTask(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("build beefy consensus task: %w", err)
	}
	if consensusResult == nil {
		log.WithField("beefyBlockNumber", beefyBlockNumber).Debug("[instant-v2] No beefy consensus update needed")
		return nil
	}
	beefyTarget := consensusResult.BeefyTarget
	mmrAnchorRelayBlock := consensusResult.MMRAnchorRelayBlock

	rewardAddress, err := li.rewardAddressBytes()
	if err != nil {
		return fmt.Errorf("convert reward address: %w", err)
	}
	relayGasPriceWei, err := li.ethereumWriter.SuggestRelayGasPrice(ctx)
	if err != nil {
		return fmt.Errorf("suggest relay gas price: %w", err)
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
		return err
	}

	if batch == nil || v2Count == 0 {
		log.WithFields(logBatchComposition(v2Count, v1Count)).
			Debug("[instant-v2] No V2 pending messages; skipping instant relay (V1-only deferred to direct-relay/handover)")
		return nil
	}
	if fiatShamirResult == nil || len(fiatShamirResult.Calldata) == 0 {
		log.WithField("beefyBlockNumber", beefyBlockNumber).Debug("[instant-v2] No beefy consensus calldata produced")
		return nil
	}

	txOpts := li.ethereumWriter.conn.MakeTxOpts(ctx)
	li.applyMulticallGasFloor(txOpts, batch.MinGasSum)
	fiatShamirCalls := append([]contracts.Multicall3Call3{{
		Target:       beefyTarget,
		AllowFailure: false,
		CallData:     fiatShamirResult.Calldata,
	}}, batch.Calls...)

	fiatShamirProfitable, fiatShamirMetrics, fiatShamirProfitabilityFeeWei, pricingMode, err :=
		li.evaluateFastPathProfitability(ctx, batch, txOpts.GasLimit, fiatShamirCalls)
	if err != nil {
		return err
	}
	if FastPathDryRun {
		applyEffectiveGasLimit(txOpts, fiatShamirMetrics)
	}

	fields := log.Fields{
		"beefyBlockNumber":            beefyBlockNumber,
		"mmrAnchorRelayBlock":         mmrAnchorRelayBlock,
		"pricingMode":                 pricingMode,
		"estimatedGas":                fiatShamirMetrics.EstimatedGas,
		"effectiveGasLimit":           txOpts.GasLimit,
		"baseFeeWei":                  fiatShamirMetrics.GasPriceWei.String(),
		"estimatedExecutionFeeWei":    fiatShamirMetrics.EstimatedFeeWei.String(),
		"conservativeExecutionFeeWei": fiatShamirMetrics.ConservativeFeeWei.String(),
		"baseFeeMarginBps":            FastPathBaseFeeMarginBps,
		"profitabilityBasisFeeWei":    fiatShamirProfitabilityFeeWei.String(),
		"actualCollectedFeeWei":       batch.TotalFeeWei.String(),
		"isFiatShamirProfitable":      fiatShamirProfitable,
	}
	for k, v := range logBatchComposition(v2Count, v1Count) {
		fields[k] = v
	}
	log.WithFields(fields).Info("[instant-v2] Evaluated fiat-shamir BEEFY profitability for fast path")

	if !fiatShamirProfitable {
		log.Info("[instant-v2] Fast-path batch is unprofitable; skipping instant relay")
		return nil
	}

	tx, err := li.submitAggregate3FastPath(ctx, txOpts, fiatShamirCalls)
	if err != nil {
		return err
	}

	submitFields := log.Fields{
		"txHash":                   tx.Hash().Hex(),
		"beefyBlockNumber":         beefyBlockNumber,
		"batchedCallCount":         len(fiatShamirCalls),
		"actualCollectedFeeWei":    batch.TotalFeeWei.String(),
		"estimatedExecutionFeeWei": fiatShamirMetrics.EstimatedFeeWei.String(),
	}
	for k, v := range logBatchComposition(v2Count, v1Count) {
		submitFields[k] = v
	}
	if txOpts.GasTipCap != nil {
		submitFields["gasTipCapWei"] = txOpts.GasTipCap.String()
	}
	log.WithFields(submitFields).Info("[instant-v2] Submitted Multicall3 aggregate3 transaction for fast path")

	return nil
}

type MessageBatchBuilder struct {
	gatewayAddress common.Address
	allowFailure   bool
	Calls          []contracts.Multicall3Call3
	MinGasSum uint64 // gas-limit floor (MaxDispatchGas for v2)
	FeeGasSum uint64 // expected burn for fee pricing
	TotalFeeWei *big.Int
}

func NewMessageBatchBuilder(gatewayAddress common.Address, allowFailure bool) *MessageBatchBuilder {
	return &MessageBatchBuilder{
		gatewayAddress: gatewayAddress,
		allowFailure:   allowFailure,
		Calls:          make([]contracts.Multicall3Call3, 0),
		TotalFeeWei:    big.NewInt(0),
	}
}

func (b *MessageBatchBuilder) AddCall(calldata []byte, minGas, feeGas uint64, feeWei *big.Int) {
	b.Calls = append(b.Calls, contracts.Multicall3Call3{
		Target:       b.gatewayAddress,
		AllowFailure: b.allowFailure,
		CallData:     calldata,
	})
	b.MinGasSum += minGas
	b.FeeGasSum += feeGas
	b.TotalFeeWei.Add(b.TotalFeeWei, feeWei)
}

func (li *BeefyInstantSyncer) checkV2ProofAlreadyRelayed(ctx context.Context, proof *MessageProof) (bool, error) {
	nonce := uint64(proof.Message.OriginalMessage.Nonce)
	isRelayed, err := li.beefyListener.scanner.isNonceRelayed(ctx, nonce)
	if err != nil {
		return false, fmt.Errorf("re-check if nonce %d is relayed: %w", proof.Message.OriginalMessage.Nonce, err)
	}
	if isRelayed {
		log.Infof("nonce %d already relayed, skipping individual v2 message", proof.Message.OriginalMessage.Nonce)
		return true, nil
	}
	return false, nil
}
