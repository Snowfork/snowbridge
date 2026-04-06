package parachain

import (
	"context"
	"fmt"
	"math/big"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"golang.org/x/sync/errgroup"
)

func minGasForV2SubmitProof(proof *MessageProof) uint64 {
	gas := BaseMessageVerificationGas
	for _, cmd := range proof.Message.OriginalMessage.Commands {
		gas += uint64(cmd.MaxDispatchGas) + gatewayDispatchOverheadBuffer
	}
	return gas
}

// applyMulticallGasFloor ensures aggregate3 has enough gas for submitFiatShamir plus nested v2_submit.
// eth_estimateGas often undershoots here because failing v2_submit (e.g. InsufficientGasLimit) is swallowed
// when AllowFailure is true, so the tx appears cheaper than a fully successful delivery.
func (li *BeefyInstantSyncer) applyMulticallGasFloor(opts *bind.TransactOpts, minV2SubmitGasSum uint64) {
	minTotal := BaseBeefyFiatShamirGas + multicall3AggregateOverheadBuffer + minV2SubmitGasSum
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

type BeefyInstantSyncer struct {
	config             *Config
	beefyListener      *BeefyListener
	beefyOnDemandRelay *beefy.OnDemandRelay
	ethereumWriter     *EthereumWriter
	multicall3         *contracts.Multicall3
	beefyClientABI     abi.ABI
}

func NewBeefyInstantSyncer(
	config *Config,
	beefyListener *BeefyListener,
	beefyOnDemandRelay *beefy.OnDemandRelay,
	ethereumWriter *EthereumWriter,
) (*BeefyInstantSyncer, error) {
	beefyClientABI, err := abi.JSON(strings.NewReader(contracts.BeefyClientABI))
	if err != nil {
		return nil, fmt.Errorf("parse beefy client ABI: %w", err)
	}

	return &BeefyInstantSyncer{
		config:             config,
		beefyListener:      beefyListener,
		beefyOnDemandRelay: beefyOnDemandRelay,
		ethereumWriter:     ethereumWriter,
		beefyClientABI:     beefyClientABI,
	}, nil
}

func (li *BeefyInstantSyncer) Start(ctx context.Context, eg *errgroup.Group) error {
	// Initialize the beefy listener to setup the scanner
	err := li.beefyListener.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize beefy listener: %w", err)
	}
	err = li.ethereumWriter.initialize()
	if err != nil {
		return fmt.Errorf("initialize ethereum writer: %w", err)
	}
	// Multicall3 must be bound after Ethereum ConnectWithHeartBeat (see InstantRelay.Start),
	// otherwise BoundContract holds a nil ethclient and Transact panics.
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
	var fetchInterval time.Duration
	if li.config.FetchInterval == 0 {
		fetchInterval = 180 * time.Second
	} else {
		fetchInterval = time.Duration(li.config.FetchInterval) * time.Second
	}

	ticker := time.NewTicker(fetchInterval)

	eg.Go(func() error {

		for {
			finalizedBeefyBlockHash, err := li.beefyListener.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
			if err != nil {
				return fmt.Errorf("fetch beefy finalized head: %w", err)
			}
			finalizedBeefyBlockHeader, err := li.beefyListener.relaychainConn.API().RPC.Chain.GetHeader(finalizedBeefyBlockHash)
			if err != nil {
				return fmt.Errorf("fetch block header: %w", err)
			}
			latestBeefyBlockNumber := uint64(finalizedBeefyBlockHeader.Number)
			err = li.doScanAndUpdate(ctx, latestBeefyBlockNumber)
			if err != nil {
				return fmt.Errorf("scan for sync tasks: %w", err)
			}

			select {
			case <-ctx.Done():
				return nil
			case <-ticker.C:
				continue
			}
		}
	})

	return nil
}

func (li *BeefyInstantSyncer) isRelayProfitable(ctx context.Context, tasks []*Task) (bool, error) {
	totalFee := new(big.Int)
	for _, task := range tasks {
		if task == nil || task.MessageProofs == nil || len(*task.MessageProofs) == 0 {
			continue
		}
		for _, messageProof := range *task.MessageProofs {
			totalFee.Add(totalFee, &messageProof.Message.Fee)
		}
	}
	gasPrice, err := li.ethereumWriter.SuggestRelayGasPrice(ctx)
	if err != nil {
		return false, fmt.Errorf("suggest gas price: %w", err)
	}
	fees := li.config.Sink.Fees
	requireFee := new(big.Int).Mul(gasPrice, new(big.Int).SetUint64(BaseBeefyFiatShamirGas))
	numerator := new(big.Int).SetUint64(fees.FeeRatioNumerator)
	denominator := new(big.Int).SetUint64(fees.FeeRatioDenominator)
	requireFee.Mul(requireFee, numerator)
	requireFee.Div(requireFee, denominator)
	for _, task := range tasks {
		if task == nil || task.MessageProofs == nil || len(*task.MessageProofs) == 0 {
			continue
		}
		for _, messageProof := range *task.MessageProofs {
			requireFee.Add(requireFee, li.ethereumWriter.MessageMinFeeWei(gasPrice, &messageProof))
		}
	}
	isProfitable := totalFee.Cmp(requireFee) >= 0
	log.WithFields(log.Fields{
		"totalFee":     totalFee.String(),
		"requireFee":   requireFee.String(),
		"isProfitable": isProfitable,
	}).Info("isProfitable")
	return isProfitable, nil
}

func (li *BeefyInstantSyncer) doScanAndUpdate(ctx context.Context, beefyBlockNumber uint64) error {
	if !li.beefyOnDemandRelay.GetConfig().Sink.EnableFiatShamir {
		return fmt.Errorf("multicall instant sync requires EnableFiatShamir")
	}

	// Scan for undelivered orders using the latest BEEFY block number on the relay chain.
	tasks, err := li.beefyListener.scanner.Scan(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("scan for sync tasks: %w", err)
	}
	if len(tasks) == 0 {
		log.Info("No tasks found, skipping")
		return nil
	}
	// Check if the relay consensus is profitable
	isProfitable, err := li.isRelayProfitable(ctx, tasks)
	if err != nil {
		return fmt.Errorf("check is relay consensus profitable: %w", err)
	}
	if !isProfitable {
		log.Info("Relay consensus is not profitable, skipping")
		return nil
	}

	log.Infof("Building Multicall3 batch for BEEFY block number %d", beefyBlockNumber)
	beefyCalldata, beefyTarget, mmrAnchorRelayBlock, err := li.beefyOnDemandRelay.BuildFiatShamirCalldata(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("build beefy consensus calldata: %w", err)
	}
	if len(beefyCalldata) == 0 {
		log.Info("Consensus update already synced or not ready, skipping")
		return nil
	}
	if mmrAnchorRelayBlock != beefyBlockNumber {
		log.WithFields(log.Fields{
			"finalizedBeefyHeadBlock": beefyBlockNumber,
			"commitmentRelayBlock":    mmrAnchorRelayBlock,
		}).Info("MMR proofs will anchor at commitment relay block (may differ from finalized head)")
	}

	rewardAddress, err := util.HexStringTo32Bytes(li.config.RewardAddress)
	if err != nil {
		return fmt.Errorf("convert reward address: %w", err)
	}

	calls := []contracts.Multicall3Call3{{
		Target:       beefyTarget,
		AllowFailure: false,
		CallData:     beefyCalldata,
	}}

	expectedMessageCalls := 0
	var minV2ProofGasSum uint64
	for _, task := range tasks {
		if task == nil || task.MessageProofs == nil || len(*task.MessageProofs) == 0 {
			continue
		}

		paraNonce := (*task.MessageProofs)[0].Message.OriginalMessage.Nonce
		isRelayed, err := li.beefyListener.scanner.isNonceRelayed(ctx, uint64(paraNonce))
		if err != nil {
			return fmt.Errorf("check if nonce %d is relayed: %w", paraNonce, err)
		}
		if isRelayed {
			log.Infof("nonce %d already relayed, skipping", paraNonce)
			continue
		}

		log.Infof("generating proof for nonce %d", paraNonce)
		task.ProofOutput, err = li.beefyListener.generateProof(ctx, task.ProofInput, task.Header, &mmrAnchorRelayBlock)
		if err != nil {
			return fmt.Errorf("generate proof for nonce %d: %w", paraNonce, err)
		}

		isRelayed, err = li.beefyListener.scanner.isNonceRelayed(ctx, uint64(paraNonce))
		if err != nil {
			return fmt.Errorf("re-check if nonce %d is relayed: %w", paraNonce, err)
		}
		if isRelayed {
			log.Infof("nonce %d was relayed by another relayer while generating proof, skipping", paraNonce)
			continue
		}

		for _, proof := range *task.MessageProofs {
			profitable, err := li.ethereumWriter.isRelayMessageProfitable(ctx, &proof)
			if err != nil {
				return fmt.Errorf("determine message profitability: %w", err)
			}
			if !profitable {
				continue
			}

			calldata, err := li.ethereumWriter.BuildV2SubmitCalldata(&proof, task.ProofOutput, rewardAddress)
			if err != nil {
				return fmt.Errorf("build v2_submit calldata for nonce %d: %w", proof.Message.OriginalMessage.Nonce, err)
			}

			calls = append(calls, contracts.Multicall3Call3{
				Target:       li.ethereumWriter.GatewayAddress(),
				AllowFailure: true,
				CallData:     calldata,
			})
			expectedMessageCalls++
			minV2ProofGasSum += minGasForV2SubmitProof(&proof)
		}
	}

	if expectedMessageCalls == 0 {
		log.Info("No profitable message deliveries to batch, skipping")
		return nil
	}

	txOpts := li.ethereumWriter.conn.MakeTxOpts(ctx)
	li.applyMulticallGasFloor(txOpts, minV2ProofGasSum)
	tx, err := li.multicall3.Aggregate3(txOpts, calls)
	if err != nil {
		return fmt.Errorf("submit multicall3 aggregate3 transaction: %w", err)
	}

	log.WithFields(log.Fields{
		"txHash":             tx.Hash().Hex(),
		"beefyBlockNumber":   beefyBlockNumber,
		"batchedCallCount":   len(calls),
		"messageCallCount":   expectedMessageCalls,
		"multicall3Contract": common.HexToAddress(li.config.Sink.Contracts.Multicall3).Hex(),
	}).Info("Submitted Multicall3 aggregate3 transaction")

	receipt, err := li.ethereumWriter.conn.WatchTransaction(ctx, tx, 1)
	if err != nil {
		return fmt.Errorf("watch multicall3 aggregate3 transaction: %w", err)
	}

	err = li.logMulticallReceipt(receipt)
	if err != nil {
		return fmt.Errorf("parse multicall receipt: %w", err)
	}

	beefyBlockSynced, _, err := li.beefyListener.fetchLatestBeefyBlock(ctx)
	if err != nil {
		return fmt.Errorf("fetch latest beefy block: %w", err)
	}
	if beefyBlockSynced < beefyBlockNumber {
		return fmt.Errorf("beefy block %d not synced to light client, recent synced %d", beefyBlockNumber, beefyBlockSynced)
	}

	return nil
}

func (li *BeefyInstantSyncer) logMulticallReceipt(receipt *gethTypes.Receipt) error {
	for _, ev := range receipt.Logs {
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
