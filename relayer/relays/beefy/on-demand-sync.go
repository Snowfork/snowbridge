package beefy

import (
	"context"
	"errors"
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
	tokenBucket       *TokenBucket
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
		sleep(ctx, time.Minute*1)
		log.Info("Starting check")

		paraNonce, ethNonce, err := relay.queryNonces(ctx)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			log.WithError(err).Error("Query nonces")
			continue
		}

		log.WithFields(log.Fields{
			"paraNonce": paraNonce,
			"ethNonce":  ethNonce,
		}).Info("Nonces checked")

		if paraNonce > ethNonce {

			// Check if we are rate-limited
			if !relay.tokenBucket.TryConsume(1) {
				log.Info("Rate-limit exceeded")
				continue
			}

			log.Info("Performing sync")

			beefyBlockHash, err := relay.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
			if err != nil {
				if errors.Is(err, context.Canceled) {
					return nil
				}
				log.WithError(err).Error("Fetch latest beefy block hash")
				continue
			}

			header, err := relay.relaychainConn.API().RPC.Chain.GetHeader(beefyBlockHash)
			if err != nil {
				if errors.Is(err, context.Canceled) {
					return nil
				}
				log.WithError(err).Error("Fetch latest beefy block header")
				continue
			}

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
}

func (relay *OnDemandRelay) waitUntilMessagesSynced(ctx context.Context, paraNonce uint64) {
	sleep(ctx, time.Minute*10)
	for {
		ethNonce, err := relay.fetchEthereumNonce(ctx)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return
			}
			log.WithError(err).Error("fetch latest ethereum nonce")
			sleep(ctx, time.Minute*1)
			continue
		}

		if ethNonce >= paraNonce {
			return
		}
	}

}

func sleep(ctx context.Context, d time.Duration) {
	select {
	case <-ctx.Done():
		return
	case <-time.After(d):
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
