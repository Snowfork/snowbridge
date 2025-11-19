package parachain

import (
	"context"
	"fmt"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"golang.org/x/sync/errgroup"
)

type BeefyInstantSyncer struct {
	beefyListener      *BeefyListener
	beefyOnDemandRelay *beefy.OnDemandRelay
}

func NewBeefyInstantSyncer(
	beefyListener *BeefyListener,
	beefyOnDemandRelay *beefy.OnDemandRelay,
) *BeefyInstantSyncer {
	return &BeefyInstantSyncer{
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

	ticker := time.NewTicker(time.Second * 60)

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

// Todo: Batch the two calls(update consensus & v2_submit) to avoid front-running
func (li *BeefyInstantSyncer) doScanAndUpdate(ctx context.Context, beefyBlockNumber uint64) error {
	// Scan for undelivered orders using the latest BEEFY block number on the relay chain.
	tasks, err := li.beefyListener.scanner.Scan(ctx, beefyBlockNumber)
	if err != nil {
		return err
	}
	if len(tasks) > 0 {
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
	}
	for _, task := range tasks {
		err = li.beefyListener.waitAndSend(ctx, task, 0)
		if err != nil {
			return fmt.Errorf("send task: %w", err)
		}
	}

	return nil
}
