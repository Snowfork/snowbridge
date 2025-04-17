package beefy

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"

	log "github.com/sirupsen/logrus"
)

type OnDemandRelay struct {
	config           *Config
	ethereumConn     *ethereum.Connection
	parachainConn    *parachain.Connection
	relaychainConn   *relaychain.Connection
	polkadotListener *PolkadotListener
	ethereumWriter   *EthereumWriter
	gatewayContract  *contracts.Gateway
	tokenBucket      *TokenBucket
}

func NewOnDemandRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*OnDemandRelay, error) {
	ethereumConn := ethereum.NewConnection(&config.Sink.Ethereum, ethereumKeypair)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	parachainConn := parachain.NewConnection(config.Source.BridgeHub.Endpoint, nil)

	polkadotListener := NewPolkadotListener(&config.Source, relaychainConn)
	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn)

	relay := OnDemandRelay{
		config:           config,
		ethereumConn:     ethereumConn,
		parachainConn:    parachainConn,
		relaychainConn:   relaychainConn,
		polkadotListener: polkadotListener,
		ethereumWriter:   ethereumWriter,
		gatewayContract:  nil,
		tokenBucket: NewTokenBucket(
			config.OnDemandSync.MaxTokens,
			config.OnDemandSync.RefillAmount,
			time.Duration(config.OnDemandSync.RefillPeriod)*time.Second,
		),
	}

	return &relay, nil
}

func (relay *OnDemandRelay) Start(ctx context.Context) error {
	err := relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("connect to ethereum: %w", err)
	}
	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return fmt.Errorf("connect to relaychain: %w", err)
	}
	err = relay.parachainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
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

	relay.tokenBucket.Start(ctx)

	for {
		log.Info("Starting check")
		sleep(ctx, time.Second*10)

		paraNonce, err := relay.fetchLatestNonce(ctx)

		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			log.WithError(err).Error("Query latest parachain nonce")
			continue
		}

		log.WithFields(log.Fields{
			"paraNonce": paraNonce,
		}).Info("Nonce checked")

		relayed, err := relay.isNonceRelayed(ctx, paraNonce)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			log.Error(fmt.Errorf("Check nonce relayed: %d, %w", paraNonce, err))
			continue
		}
		if relayed {
			continue
		}

		paraBlock, err := relay.fetchParachainBlockByNonce(ctx, paraNonce)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			log.Error(fmt.Errorf("fetch paraBlock of the nonce: %d, %w", paraNonce, err))
			continue
		}

		inclusionBlock, err := relay.fetchRelaychainInclusionBlock(paraBlock)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			log.Error(fmt.Errorf("fetch relayBlock of the nonce: %d, paraBlock: %d, error: %w", paraNonce, paraBlock, err))
			continue
		}

		log.WithFields(log.Fields{
			"paraNonce":  paraNonce,
			"paraBlock":  paraBlock,
			"relayBlock": inclusionBlock,
		}).Info("find relaychain block which includes the parachain order")

		var header *types.Header

		for {
			beefyBlockHash, err := relay.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
			if err != nil {
				if errors.Is(err, context.Canceled) {
					return nil
				}
				log.WithError(err).Error("Fetch latest beefy block hash")
				continue
			}
			header, err = relay.relaychainConn.API().RPC.Chain.GetHeader(beefyBlockHash)
			if err != nil {
				if errors.Is(err, context.Canceled) {
					return nil
				}
				log.WithError(err).Error("Fetch latest beefy block header")
				continue
			}
			if uint64(header.Number) > inclusionBlock {
				break
			}
			time.Sleep(10 * time.Second)
		}

		// Check if we are rate-limited
		if !relay.tokenBucket.TryConsume(1) {
			log.Info("Rate-limit exceeded")
			continue
		}

		log.Info("Performing sync")

		err = relay.sync(ctx, uint64(header.Number))
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			log.WithError(err).Error("Sync failed")
			continue
		}

		log.Info("Sync completed")

		relay.waitUntilMessagesSynced(ctx, paraNonce)
	}
}

func (relay *OnDemandRelay) waitUntilMessagesSynced(ctx context.Context, paraNonce uint64) {
	waitingTime := 0
	for {
		log.Info(fmt.Sprintf("waiting for nonce %d picked by parachain relayer", paraNonce))
		relayed, err := relay.isNonceRelayed(ctx, paraNonce)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return
			}
			log.WithError(err).Error("check nonce relayed")
			continue
		}
		waitingTime++

		if relayed || waitingTime > 10 {
			break
		}
		time.Sleep(10 * time.Second)
	}
}

func sleep(ctx context.Context, d time.Duration) {
	select {
	case <-ctx.Done():
		return
	case <-time.After(d):
	}
}

func (relay *OnDemandRelay) fetchLatestNonce(_ context.Context) (uint64, error) {
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
	if paraOutboundNonce == 0 {
		return paraOutboundNonce, nil
	}

	return paraOutboundNonce - 1, nil
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
	err := relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("connect to ethereum: %w", err)
	}
	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return fmt.Errorf("connect to relaychain: %w", err)
	}
	err = relay.parachainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
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

func (relay *OnDemandRelay) fetchParachainBlockByNonce(_ context.Context, nonce uint64) (uint64, error) {
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

func (relay *OnDemandRelay) isNonceRelayed(ctx context.Context, nonce uint64) (bool, error) {
	isRelayed, err := relay.gatewayContract.V2IsDispatched(&bind.CallOpts{
		Pending: true,
		Context: ctx,
	}, nonce)
	if err != nil {
		return false, fmt.Errorf("check nonce from gateway contract: %w", err)
	}
	return isRelayed, nil
}
