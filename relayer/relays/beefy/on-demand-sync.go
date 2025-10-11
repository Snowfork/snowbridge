package beefy

import (
	"context"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

type OnDemandRelay struct {
	config            *Config
	ethereumConn      *ethereum.Connection
	parachainConn     *parachain.Connection
	relaychainConn    *relaychain.Connection
	polkadotListener  *PolkadotListener
	ethereumWriter    *EthereumWriter
	gatewayContract   *contracts.Gateway
	assetHubChannelID [32]byte
	activeTasks       TaskMap
}

func NewOnDemandRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*OnDemandRelay, error) {
	ethereumConn := ethereum.NewConnection(&config.Sink.Ethereum, ethereumKeypair)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	parachainConn := parachain.NewConnection(config.Source.BridgeHub.Endpoint, nil)

	polkadotListener := NewPolkadotListener(&config.Source, relaychainConn)
	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn)

	assetHubChannelID, err := types.HexDecodeString(config.OnDemandSync.AssetHubChannelID)
	if err != nil {
		return nil, fmt.Errorf("hex decode assethub channel: %w", err)
	}

	relay := OnDemandRelay{
		config:            config,
		ethereumConn:      ethereumConn,
		parachainConn:     parachainConn,
		relaychainConn:    relaychainConn,
		polkadotListener:  polkadotListener,
		ethereumWriter:    ethereumWriter,
		gatewayContract:   nil,
		assetHubChannelID: *(*[32]byte)(assetHubChannelID),
		activeTasks:       *NewTaskMap(config.OnDemandSync.MaxTasks, config.OnDemandSync.MergePeriod),
	}

	return &relay, nil
}

func (relay *OnDemandRelay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.ethereumConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Sink.Ethereum.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to ethereum: %w", err)
	}
	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.Polkadot.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to relaychain: %w", err)
	}
	err = relay.parachainConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.BridgeHub.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to parachain: %w", err)
	}
	err = relay.ethereumWriter.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize EthereumWriter: %w", err)
	}

	gatewayAddress := common.HexToAddress(relay.config.Sink.Contracts.Gateway)
	gatewayContract, err := contracts.NewGateway(gatewayAddress, relay.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("create gateway client: %w", err)
	}
	relay.gatewayContract = gatewayContract

	enqueueTicker := time.NewTicker(time.Second * 120)

	eg.Go(func() error {
		defer enqueueTicker.Stop()
		for {
			log.Info("Starting check nonces")

			paraNonce, ethNonce, err := relay.queryNonces(ctx)
			if err != nil {
				return fmt.Errorf("Query nonces: %w", err)
			}

			log.WithFields(log.Fields{
				"paraNonce": paraNonce,
				"ethNonce":  ethNonce,
			}).Info("Nonces checked")

			if paraNonce > ethNonce {
				err = relay.queue(ctx, paraNonce)
				if err != nil {
					return fmt.Errorf("Queue failed: %w", err)
				}
			}
			select {
			case <-ctx.Done():
				return nil
			case <-enqueueTicker.C:
				continue
			}
		}
	})

	scheduleTicker := time.NewTicker(time.Second * 60)
	eg.Go(func() error {
		defer scheduleTicker.Stop()
		for {
			log.Info("Scheduling pending nonces")
			tasks := relay.activeTasks.InspectAll()
			log.WithFields(log.Fields{
				"pendingTasks": len(tasks),
				"lastUpdate":   time.Unix(int64(relay.activeTasks.lastUpdated), 0),
			}).Info("Queue info")
			for _, task := range tasks {
				log.WithFields(log.Fields{
					"nonce":      task.nonce,
					"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
					"status":     task.status,
					"skipped":    task.req.Skippable,
					"timestamp":  time.Unix(int64(task.timestamp), 0),
				}).Info("Task info")
			}
			if len(tasks) > 0 {
				err = relay.schedule(ctx, eg)
				if err != nil {
					return fmt.Errorf("Schedule failed: %w", err)
				}
			}
			select {
			case <-ctx.Done():
				return nil
			case <-scheduleTicker.C:
				continue
			}
		}
	})
	return nil
}

func (relay *OnDemandRelay) waitUntilMessagesSynced(ctx context.Context, paraNonce uint64) error {
	var cnt uint64
	for {
		ethNonce, err := relay.fetchEthereumNonce(ctx)

		if err == nil && ethNonce >= paraNonce {
			return nil
		}
		time.Sleep(time.Second * 30)
		cnt++
		if cnt > 10 {
			return fmt.Errorf("timeout waiting for messages to be relayed")
		}
	}
}

func (relay *OnDemandRelay) queryNonces(ctx context.Context) (uint64, uint64, error) {
	paraNonce, err := relay.fetchLatestParachainNonce(ctx)
	if err != nil {
		return 0, 0, fmt.Errorf("fetch latest parachain nonce: %w", err)
	}

	ethNonce, err := relay.fetchEthereumNonce(ctx)
	if err != nil {
		return 0, 0, fmt.Errorf("fetch latest ethereum nonce: %w", err)
	}

	return paraNonce, ethNonce, nil
}

func (relay *OnDemandRelay) fetchLatestParachainNonce(_ context.Context) (uint64, error) {
	paraNonceKey, err := types.CreateStorageKey(
		relay.parachainConn.Metadata(), "EthereumOutboundQueue", "Nonce",
		relay.assetHubChannelID[:], nil,
	)
	if err != nil {
		return 0, fmt.Errorf(
			"create storage key for EthereumOutboundQueue.Nonce(%v): %w",
			Hex(relay.assetHubChannelID[:]), err,
		)
	}
	var paraOutboundNonce uint64
	ok, err := relay.parachainConn.API().RPC.State.GetStorageLatest(paraNonceKey, &paraOutboundNonce)
	if err != nil {
		return 0, fmt.Errorf(
			"fetch storage EthereumOutboundQueue.Nonce(%v): %w",
			Hex(relay.assetHubChannelID[:]), err,
		)
	}
	if !ok {
		paraOutboundNonce = 0
	}

	return paraOutboundNonce, nil
}

func (relay *OnDemandRelay) fetchEthereumNonce(ctx context.Context) (uint64, error) {
	opts := bind.CallOpts{
		Context: ctx,
	}
	ethInboundNonce, _, err := relay.gatewayContract.ChannelNoncesOf(&opts, relay.assetHubChannelID)
	if err != nil {
		return 0, fmt.Errorf(
			"fetch Gateway.ChannelNoncesOf(%v): %w",
			Hex(relay.assetHubChannelID[:]), err,
		)
	}

	return ethInboundNonce, nil
}

func (relay *OnDemandRelay) sync(ctx context.Context, blockNumber uint64) error {
	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}
	// Ignore relay block already synced
	if blockNumber <= state.LatestBeefyBlock {
		log.WithFields(log.Fields{
			"validatorSetID": state.CurrentValidatorSetID,
			"beefyBlock":     state.LatestBeefyBlock,
			"relayBlock":     blockNumber,
		}).Info("Relay block already synced, just ignore")
		return nil
	}

	// generate beefy update for that specific relay block
	task, err := relay.polkadotListener.generateBeefyUpdate(blockNumber)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}

	// Ignore commitment earlier than LatestBeefyBlock which is outdated
	if task.SignedCommitment.Commitment.BlockNumber <= uint32(state.LatestBeefyBlock) {
		log.WithFields(log.Fields{
			"latestBeefyBlock":      state.LatestBeefyBlock,
			"currentValidatorSetID": state.CurrentValidatorSetID,
			"nextValidatorSetID":    state.NextValidatorSetID,
			"blockNumberToSync":     task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment outdated, just ignore")
		return nil
	}
	if task.SignedCommitment.Commitment.ValidatorSetID > state.NextValidatorSetID {
		log.WithFields(log.Fields{
			"latestBeefyBlock":      state.LatestBeefyBlock,
			"currentValidatorSetID": state.CurrentValidatorSetID,
			"nextValidatorSetID":    state.NextValidatorSetID,
			"validatorSetIDToSync":  task.SignedCommitment.Commitment.ValidatorSetID,
		}).Warn("Task unexpected, wait for mandatory updates to catch up first")
		return nil
	}

	// Submit the task
	if task.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		task.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		task.ValidatorsRoot = state.NextValidatorSetRoot
	}
	err = relay.ethereumWriter.submit(ctx, task)
	if err != nil {
		return fmt.Errorf("fail to submit beefy update: %w", err)
	}

	updatedState, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}
	log.WithFields(log.Fields{
		"latestBeefyBlock":      updatedState.LatestBeefyBlock,
		"currentValidatorSetID": updatedState.CurrentValidatorSetID,
		"nextValidatorSetID":    updatedState.NextValidatorSetID,
	}).Info("Sync beefy update success")
	return nil
}

func (relay *OnDemandRelay) OneShotStart(ctx context.Context, beefyBlockNumber uint64) error {
	eg, ctx := errgroup.WithContext(ctx)
	err := relay.ethereumConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Sink.Ethereum.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to ethereum: %w", err)
	}
	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.Polkadot.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to relaychain: %w", err)
	}
	err = relay.parachainConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.BridgeHub.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to parachain: %w", err)
	}
	err = relay.ethereumWriter.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize EthereumWriter: %w", err)
	}

	gatewayAddress := common.HexToAddress(relay.config.Sink.Contracts.Gateway)
	gatewayContract, err := contracts.NewGateway(gatewayAddress, relay.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("create gateway client: %w", err)
	}
	relay.gatewayContract = gatewayContract

	log.Info("Performing sync")

	if beefyBlockNumber == 0 {
		beefyBlockHash, err := relay.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return fmt.Errorf("Fetch latest beefy block hash: %w", err)
		}

		header, err := relay.relaychainConn.API().RPC.Chain.GetHeader(beefyBlockHash)
		if err != nil {
			return fmt.Errorf("Fetch latest beefy block header: %w", err)
		}
		beefyBlockNumber = uint64(header.Number)
	}

	err = relay.sync(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("Sync failed: %w", err)
	}

	log.Info("Sync completed")
	return nil
}

func (relay *OnDemandRelay) queue(ctx context.Context, nonce uint64) error {
	if relay.activeTasks.Full() {
		log.Info("Task queue full, wait for scheduling")
		return nil
	}
	_, ok := relay.activeTasks.Load(nonce)
	if ok {
		log.WithFields(log.Fields{
			"nonce": nonce,
		}).Info("nonce in syncing, just ignore")
		return nil
	}
	log.WithFields(log.Fields{
		"nonce": nonce,
	}).Info("Performing queueing")

	// sleep to ensure that beefy head newer than relay chain block in which the parachain block was accepted.
	time.Sleep(time.Second * 90)

	beefyBlockHash, err := relay.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
	if err != nil {
		return fmt.Errorf("Fetch latest beefy block hash: %w", err)
	}
	header, err := relay.relaychainConn.API().RPC.Chain.GetHeader(beefyBlockHash)
	if err != nil {
		return fmt.Errorf("Fetch latest beefy block header: %w", err)
	}
	beefyBlockNumber := uint64(header.Number)
	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}
	// Ignore relay block already synced
	if beefyBlockNumber <= state.LatestBeefyBlock {
		log.WithFields(log.Fields{
			"validatorSetID": state.CurrentValidatorSetID,
			"beefyBlock":     state.LatestBeefyBlock,
			"relayBlock":     header.Number,
		}).Info("Relay block already synced, just ignore")
		return nil
	}

	// generate beefy update for that specific relay block
	task, err := relay.polkadotListener.generateBeefyUpdate(beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}

	// Ignore commitment earlier than LatestBeefyBlock which is outdated
	if task.SignedCommitment.Commitment.BlockNumber <= uint32(state.LatestBeefyBlock) {
		log.WithFields(log.Fields{
			"latestBeefyBlock":      state.LatestBeefyBlock,
			"currentValidatorSetID": state.CurrentValidatorSetID,
			"nextValidatorSetID":    state.NextValidatorSetID,
			"blockNumberToSync":     task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment outdated, just ignore")
		return nil
	}
	if task.SignedCommitment.Commitment.ValidatorSetID > state.NextValidatorSetID {
		log.WithFields(log.Fields{
			"latestBeefyBlock":      state.LatestBeefyBlock,
			"currentValidatorSetID": state.CurrentValidatorSetID,
			"nextValidatorSetID":    state.NextValidatorSetID,
			"validatorSetIDToSync":  task.SignedCommitment.Commitment.ValidatorSetID,
		}).Warn("Task unexpected, wait for mandatory updates to catch up first")
		return nil
	}

	// Submit the task
	if task.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		task.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		task.ValidatorsRoot = state.NextValidatorSetRoot
	}
	relay.activeTasks.Store(nonce, &task)
	log.WithFields(log.Fields{
		"nonce":      nonce,
		"commitment": task.SignedCommitment.Commitment.BlockNumber,
	}).Info("Task enqueued")
	return nil
}

func (relay *OnDemandRelay) schedule(ctx context.Context, eg *errgroup.Group) error {
	task := relay.activeTasks.Pop()
	if task == nil {
		log.Info("No task available, waiting for new tasks to be queued or for the ongoing task to complete")
		return nil
	}
	// Try to merge the previous tasks
	relay.activeTasks.Merge(task.nonce)
	err := relay.activeTasks.sem.Acquire(ctx, 1)
	if err != nil {
		return err
	}
	eg.Go(func() error {
		defer relay.activeTasks.sem.Release(1)
		log.WithFields(log.Fields{
			"nonce":      task.nonce,
			"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
		}).Info("Starting beefy sync")
		err := relay.sync(ctx, uint64(task.req.SignedCommitment.Commitment.BlockNumber))
		if err != nil {
			log.WithFields(log.Fields{
				"nonce":      task.nonce,
				"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
				"error":      err,
			}).Error("Sync beefy failed")
			relay.activeTasks.SetStatus(task.nonce, TaskFailed)
		} else {
			if task.req.Skippable {
				log.WithFields(log.Fields{
					"nonce":      task.nonce,
					"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
				}).Warn("Sync beefy skipped")
				relay.activeTasks.SetStatus(task.nonce, TaskCanceled)
			} else {
				log.WithFields(log.Fields{
					"nonce":      task.nonce,
					"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
				}).Info("Sync beefy completed")
				relay.activeTasks.SetLastUpdated(task.nonce)
				err = relay.waitUntilMessagesSynced(ctx, task.nonce)
				if err != nil {
					log.WithFields(log.Fields{
						"nonce":      task.nonce,
						"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
					}).Warn("Sync beefy completed, but the parachain relay failed to sync the pending nonce in time.")
					relay.activeTasks.SetStatus(task.nonce, TaskCompleted)
				} else {
					relay.activeTasks.Delete(task.nonce)
				}
			}
		}
		return nil
	})
	return nil
}
