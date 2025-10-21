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
		activeTasks:       *NewTaskMap(config.OnDemandSync.MaxTasks, config.OnDemandSync.MergePeriod, config.OnDemandSync.ExpiredPeriod),
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

	ticker := time.NewTicker(time.Second * 60)

	eg.Go(func() error {
		defer ticker.Stop()
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
			case <-ticker.C:
				continue
			}
		}
	})

	scheduleTicker := time.NewTicker(time.Second * 60)
	eg.Go(func() error {
		defer scheduleTicker.Stop()
		for {
			log.Info("Scheduling pending nonces")
			err = relay.schedule(ctx, eg)
			if err != nil {
				return fmt.Errorf("Schedule failed: %w", err)
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
	task, err := relay.polkadotListener.generateBeefyUpdate(blockNumber)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}
	err = relay.syncBeefyUpdate(ctx, &task)
	if err != nil {
		return fmt.Errorf("Sync beefy request failed: %w", err)
	}
	return nil
}

func (relay *OnDemandRelay) syncBeefyUpdate(ctx context.Context, task *Request) error {
	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}
	logger := log.WithFields(log.Fields{
		"commitmentBlock":       task.SignedCommitment.Commitment.BlockNumber,
		"latestBeefyBlock":      state.LatestBeefyBlock,
		"currentValidatorSetID": state.CurrentValidatorSetID,
		"nextValidatorSetID":    state.NextValidatorSetID,
	})
	// Ignore commitment earlier than LatestBeefyBlock which is outdated
	if uint64(task.SignedCommitment.Commitment.BlockNumber) <= state.LatestBeefyBlock {
		logger.Info("Commitment outdated, just ignore")
		return nil
	}

	if task.SignedCommitment.Commitment.ValidatorSetID > state.NextValidatorSetID {
		logger.Warn("Task unexpected, wait for mandatory updates to catch up first")
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
	logger.WithFields(log.Fields{
		"commitmentBlock":           task.SignedCommitment.Commitment.BlockNumber,
		"updatedBeefyBlock":         updatedState.LatestBeefyBlock,
		"updatedValidatorSetID":     updatedState.CurrentValidatorSetID,
		"updatedNextValidatorSetID": updatedState.NextValidatorSetID,
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

	err = relay.sync(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("Sync failed: %w", err)
	}

	log.Info("Sync completed")
	return nil
}

// Enqueue a nonce task into the queue
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

	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}

	// Generate a Beefy request for the latest finalized block (Specify zero for latest)
	req, err := relay.polkadotListener.generateBeefyUpdate(0)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}

	logger := log.WithFields(log.Fields{
		"latestBeefyBlock":         state.LatestBeefyBlock,
		"currentValidatorSetID":    state.CurrentValidatorSetID,
		"nextValidatorSetID":       state.NextValidatorSetID,
		"commitmentBlock":          req.SignedCommitment.Commitment.BlockNumber,
		"commitmentValidatorSetID": req.SignedCommitment.Commitment.ValidatorSetID,
		"nonce":                    nonce,
	})

	// Ignore commitment earlier than LatestBeefyBlock
	if req.SignedCommitment.Commitment.BlockNumber <= uint32(state.LatestBeefyBlock) {
		logger.Info("Commitment outdated, just ignore")
		return nil
	}
	if req.SignedCommitment.Commitment.ValidatorSetID > state.NextValidatorSetID {
		logger.Warn("Task unexpected, wait for mandatory updates to catch up first")
		return nil
	}

	if req.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		req.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		req.ValidatorsRoot = state.NextValidatorSetRoot
	}
	ok = relay.activeTasks.Store(nonce, &req)
	if ok {
		logger.Info("Task enqueued")
	} else {
		logger.Warn("Task not enqueued because the queue is full")
	}
	return nil
}

// Schedule an available task for execution
func (relay *OnDemandRelay) schedule(ctx context.Context, eg *errgroup.Group) error {
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
	task := relay.activeTasks.Pop()
	if task == nil {
		log.Info("No task available, waiting for new tasks to be queued or for ongoing tasks to complete")
		return nil
	}
	err := relay.activeTasks.sem.Acquire(ctx, 1)
	if err != nil {
		return fmt.Errorf("Acquires the semaphore: %w", err)
	}
	logger := log.WithFields(log.Fields{
		"nonce":      task.nonce,
		"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
	})
	eg.Go(func() error {
		defer relay.activeTasks.sem.Release(1)
		logger.Info("Starting beefy sync")
		err := relay.syncBeefyUpdate(ctx, task.req)
		if err != nil {
			logger.Error(fmt.Sprintf("Sync beefy failed, %v", err))
			relay.activeTasks.SetStatus(task.nonce, Failed)
		} else {
			if task.req.Skippable {
				logger.Info("Sync beefy skipped")
				relay.activeTasks.SetStatus(task.nonce, Canceled)
			} else {
				logger.Info("Sync beefy completed")
				relay.activeTasks.SetLastUpdated(task.nonce)
				err = relay.waitUntilMessagesSynced(ctx, task.nonce)
				if err != nil {
					logger.Warn("Beefy sync completed, but pending nonce not synced in time")
					relay.activeTasks.SetStatus(task.nonce, Completed)
				} else {
					relay.activeTasks.Delete(task.nonce)
				}
			}
		}
		return nil
	})
	return nil
}
