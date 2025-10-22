package beefy

import (
	"bytes"
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	contractV1 "github.com/snowfork/snowbridge/relayer/contracts/v1"
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
	gatewayContractV2 *contracts.Gateway
	gatewayContractV1 *contractV1.Gateway
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
	gatewayContractV2, err := contracts.NewGateway(gatewayAddress, relay.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("create gateway client: %w", err)
	}
	relay.gatewayContractV2 = gatewayContractV2

	gatewayContractV1, err := contractV1.NewGateway(gatewayAddress, relay.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("create gateway client: %w", err)
	}
	relay.gatewayContractV1 = gatewayContractV1

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
	ethInboundNonce, _, err := relay.gatewayContractV1.ChannelNoncesOf(&opts, relay.assetHubChannelID)
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

func (relay *OnDemandRelay) syncV2(ctx context.Context) error {
	paraNonce, err := relay.fetchLatestV2Nonce(ctx)

	if err != nil {
		return fmt.Errorf("Query latest parachain nonce: %w", err)
	}
	if paraNonce == 0 {
		return nil
	}

	log.WithFields(log.Fields{
		"paraNonce": paraNonce,
	}).Info("V2 nonce checked")

	relayed, err := relay.isV2NonceRelayed(ctx, paraNonce)
	if err != nil {
		return fmt.Errorf("Check v2 nonce relayed: %w", err)
	}
	if relayed {
		return nil
	}

	paraBlock, err := relay.fetchParachainBlockByV2Nonce(ctx, paraNonce)
	if err != nil {
		return fmt.Errorf("Fetch paraBlock of v2 nonce: %w", err)
	}

	inclusionBlock, err := relay.fetchRelaychainInclusionBlock(paraBlock)
	if err != nil {
		return fmt.Errorf("Fetch relayBlock of v2 nonce: %d, paraBlock: %d, error: %w", paraNonce, paraBlock, err)
	}

	log.WithFields(log.Fields{
		"paraNonce":  paraNonce,
		"paraBlock":  paraBlock,
		"relayBlock": inclusionBlock,
	}).Info("find relaychain block which includes the pending order")

	var header *types.Header

	for {
		beefyBlockHash, err := relay.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return fmt.Errorf("Fetch latest beefy block hash: %w", err)
		}
		header, err = relay.relaychainConn.API().RPC.Chain.GetHeader(beefyBlockHash)
		if err != nil {
			return fmt.Errorf("Fetch latest beefy block header: %w", err)
		}
		if uint64(header.Number) > inclusionBlock {
			break
		}
		time.Sleep(10 * time.Second)
	}

	log.Info("Performing v2 sync")

	err = relay.sync(ctx, uint64(header.Number))
	if err != nil {
		return fmt.Errorf("Sync failed: %w", err)
	}

	log.Info("Sync completed")

	relay.waitUntilV2MessagesSynced(ctx, paraNonce)
	return nil
}

func (relay *OnDemandRelay) waitUntilV2MessagesSynced(ctx context.Context, paraNonce uint64) {
	for {
		log.Info(fmt.Sprintf("waiting for nonce %d picked by parachain relayer", paraNonce))
		relayed, err := relay.isV2NonceRelayed(ctx, paraNonce)
		if err != nil {
			log.WithError(err).Error("check nonce relayed")
			return
		}

		if relayed {
			break
		}
		time.Sleep(10 * time.Second)
	}
}

func (relay *OnDemandRelay) fetchLatestV2Nonce(_ context.Context) (uint64, error) {
	paraNonceKey, err := types.CreateStorageKey(
		relay.parachainConn.Metadata(), "EthereumOutboundQueueV2", "Nonce",
		nil,
	)
	if err != nil {
		return 0, fmt.Errorf(
			"create storage key for EthereumOutboundQueueV2.Nonce: %w",
			err,
		)
	}
	var paraOutboundNonce uint64
	ok, err := relay.parachainConn.API().RPC.State.GetStorageLatest(paraNonceKey, &paraOutboundNonce)
	if err != nil {
		return 0, fmt.Errorf(
			"fetch storage EthereumOutboundQueue.Nonce: %w",
			err,
		)
	}
	if !ok {
		paraOutboundNonce = 0
	}

	return paraOutboundNonce, nil
}

func (relay *OnDemandRelay) fetchParachainBlockByV2Nonce(_ context.Context, nonce uint64) (uint64, error) {
	nonceKey, _ := types.EncodeToBytes(types.NewU64(nonce))
	storageKey, err := types.CreateStorageKey(relay.parachainConn.Metadata(), "EthereumOutboundQueueV2", "PendingOrders", nonceKey, nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key for pendingOrder: %w", err)
	}

	var order parachain.PendingOrder
	value, err := relay.parachainConn.API().RPC.State.GetStorageRawLatest(storageKey)
	if err != nil {
		return 0, fmt.Errorf("fetch value of pendingOrder with key '%v': %w", storageKey, err)
	}
	decoder := scale.NewDecoder(bytes.NewReader(*value))
	err = decoder.Decode(&order)
	if err != nil {
		return 0, fmt.Errorf("decode order error: %w", err)
	}
	return uint64(order.BlockNumber), nil
}

func (relay *OnDemandRelay) fetchRelaychainInclusionBlock(
	paraBlockNumber uint64,
) (uint64, error) {
	validationDataKey, err := types.CreateStorageKey(relay.parachainConn.Metadata(), "ParachainSystem", "ValidationData", nil, nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key: %w", err)
	}

	paraBlockHash, err := relay.parachainConn.API().RPC.Chain.GetBlockHash(paraBlockNumber)
	if err != nil {
		return 0, fmt.Errorf("fetch parachain block hash: %w", err)
	}

	var validationData parachain.PersistedValidationData
	ok, err := relay.parachainConn.API().RPC.State.GetStorage(validationDataKey, &validationData, paraBlockHash)
	if err != nil {
		return 0, fmt.Errorf("fetch PersistedValidationData for block %v: %w", paraBlockHash.Hex(), err)
	}
	if !ok {
		return 0, fmt.Errorf("PersistedValidationData not found for block %v", paraBlockHash.Hex())
	}

	// fetch ParaId
	paraIDKey, err := types.CreateStorageKey(relay.parachainConn.Metadata(), "ParachainInfo", "ParachainId", nil, nil)
	if err != nil {
		return 0, err
	}
	var paraID uint32
	ok, err = relay.parachainConn.API().RPC.State.GetStorageLatest(paraIDKey, &paraID)
	if err != nil {
		return 0, fmt.Errorf("fetch parachain id: %w", err)
	}

	startBlock := validationData.RelayParentNumber + 1
	for i := validationData.RelayParentNumber + 1; i < startBlock+relaychain.FinalizationTimeout; i++ {
		relayBlockHash, err := relay.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(i))
		if err != nil {
			return 0, fmt.Errorf("fetch relaychain block hash: %w", err)
		}

		var paraHead types.Header
		ok, err := relay.relaychainConn.FetchParachainHead(relayBlockHash, paraID, &paraHead)
		if err != nil {
			return 0, fmt.Errorf("fetch head for parachain %v at block %v: %w", paraID, relayBlockHash.Hex(), err)
		}
		if !ok {
			return 0, fmt.Errorf("parachain %v is not registered", paraID)
		}

		if paraBlockNumber == uint64(paraHead.Number) {
			return uint64(i), nil
		}
	}

	return 0, fmt.Errorf("can't find inclusion block")
}

func (relay *OnDemandRelay) isV2NonceRelayed(ctx context.Context, nonce uint64) (bool, error) {
	isRelayed, err := relay.gatewayContractV2.V2IsDispatched(&bind.CallOpts{
		Pending: true,
		Context: ctx,
	}, nonce)
	if err != nil {
		return false, fmt.Errorf("check nonce from gateway contract: %w", err)
	}
	return isRelayed, nil
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
	gatewayContract, err := contractV1.NewGateway(gatewayAddress, relay.ethereumConn.Client())
	if err != nil {
		return fmt.Errorf("create gateway client: %w", err)
	}
	relay.gatewayContractV1 = gatewayContract

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
			"nonce":      task.id,
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
		"nonce":      task.id,
		"commitment": task.req.SignedCommitment.Commitment.BlockNumber,
	})
	eg.Go(func() error {
		defer relay.activeTasks.sem.Release(1)
		logger.Info("Starting beefy sync")
		err := relay.syncBeefyUpdate(ctx, task.req)
		if err != nil {
			logger.Error(fmt.Sprintf("Sync beefy failed, %v", err))
			relay.activeTasks.SetStatus(task.id, Failed)
		} else {
			if task.req.Skippable {
				logger.Info("Sync beefy skipped")
				relay.activeTasks.SetStatus(task.id, Canceled)
			} else {
				logger.Info("Sync beefy completed")
				relay.activeTasks.SetLastUpdated(task.id)
				err = relay.waitUntilMessagesSynced(ctx, task.id)
				if err != nil {
					logger.Warn("Beefy sync completed, but pending nonce not synced in time")
					relay.activeTasks.SetStatus(task.id, Completed)
				} else {
					relay.activeTasks.Delete(task.id)
				}
			}
		}
		return nil
	})
	return nil
}

func (relay *OnDemandRelay) isV2Enabled(ctx context.Context) bool {
	_, err := relay.fetchLatestV2Nonce(ctx)

	if err != nil {
		return !strings.Contains(err.Error(), "EthereumOutboundQueueV2 not found")
	}
	return true
}
