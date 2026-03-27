package parachain

import (
	"context"
	"fmt"
	"math/big"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"golang.org/x/sync/errgroup"
)

type BeefyInstantSyncer struct {
	config             *Config
	beefyListener      *BeefyListener
	beefyOnDemandRelay *beefy.OnDemandRelay
}

func NewBeefyInstantSyncer(
	config *Config,
	beefyListener *BeefyListener,
	beefyOnDemandRelay *beefy.OnDemandRelay,
) *BeefyInstantSyncer {
	return &BeefyInstantSyncer{
		config:             config,
		beefyListener:      beefyListener,
		beefyOnDemandRelay: beefyOnDemandRelay,
	}
}

// Todo: consider using subscription to listen for new finalized beefy headers
func (li *BeefyInstantSyncer) Start(ctx context.Context, eg *errgroup.Group) error {
	// Initialize the beefy listener to setup the scanner
	err := li.beefyListener.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize beefy listener: %w", err)
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

func (li *BeefyInstantSyncer) isRelayConsensusProfitable(ctx context.Context, tasks []*Task) (bool, error) {
	totalFee := new(big.Int)
	for _, task := range tasks {
		if task == nil || task.MessageProofs == nil || len(*task.MessageProofs) == 0 {
			continue
		}
		for _, messageProof := range *task.MessageProofs {
			totalFee.Add(totalFee, &messageProof.Message.Fee)
		}
	}
	gasPrice, err := li.beefyListener.ethereumConn.Client().SuggestGasPrice(ctx)
	if err != nil {
		return false, fmt.Errorf("suggest gas price: %w", err)
	}
	var requireFee *big.Int
	if li.beefyOnDemandRelay.GetConfig().Sink.EnableFiatShamir {
		requireFee = new(big.Int).Mul(gasPrice, new(big.Int).SetUint64(li.config.Sink.Fees.BaseBeefyFiatShamirGas))
	} else {
		requireFee = new(big.Int).Mul(gasPrice, new(big.Int).SetUint64(li.config.Sink.Fees.BaseBeefyTwoPhaseCommitGas))
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
	isProfitable, err := li.isRelayConsensusProfitable(ctx, tasks)
	if err != nil {
		return fmt.Errorf("check is relay consensus profitable: %w", err)
	}
	if !isProfitable {
		log.Info("Relay consensus is not profitable, skipping")
		return nil
	}
	// Oneshot sync with FiatShamir and ensure light client is synced to the BEEFY block number
	// before submitting any messages to the parachain
	// This is to ensure the light client has the necessary BEEFY proofs
	// to verify the parachain headers being submitted
	log.Info(fmt.Sprintf("Syncing light client to BEEFY block number %d\n", beefyBlockNumber))
	err = li.beefyOnDemandRelay.OneShotStart(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("sync beefy update on demand: %w", err)
	}
	beefyBlockSynced, _, err := li.beefyListener.fetchLatestBeefyBlock(ctx)
	if err != nil {
		return fmt.Errorf("fetch latest beefy block: %w", err)
	}
	if beefyBlockSynced < beefyBlockNumber {
		return fmt.Errorf("beefy block %d not synced to light client, recent synced %d", beefyBlockNumber, beefyBlockSynced)
	}
	for _, task := range tasks {
		err = li.beefyListener.sendTask(ctx, task)
		if err != nil {
			return fmt.Errorf("send task: %w", err)
		}
	}

	return nil
}
