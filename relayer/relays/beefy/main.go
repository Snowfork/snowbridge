package beefy

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/errgroup"

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

type Relay struct {
	config           *Config
	relaychainConn   *relaychain.Connection
	ethereumConn     *ethereum.Connection
	polkadotListener *PolkadotListener
	ethereumWriter   *EthereumWriter
}

func NewRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*Relay, error) {
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	ethereumConn := ethereum.NewConnection(&config.Sink.Ethereum, ethereumKeypair)

	polkadotListener := NewPolkadotListener(
		&config.Source,
		relaychainConn,
	)

	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn)

	log.Info("Beefy relay created")

	return &Relay{
		config:           config,
		relaychainConn:   relaychainConn,
		ethereumConn:     ethereumConn,
		polkadotListener: polkadotListener,
		ethereumWriter:   ethereumWriter,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.relaychainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
	}
	err = relay.ethereumWriter.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize ethereum writer: %w", err)
	}

	initialState, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("fetch BeefyClient current state: %w", err)
	}
	log.WithFields(log.Fields{
		"beefyBlock":     initialState.LatestBeefyBlock,
		"validatorSetID": initialState.CurrentValidatorSetID,
	}).Info("Retrieved current BeefyClient state")

	requests, err := relay.polkadotListener.Start(ctx, eg, initialState.LatestBeefyBlock, initialState.CurrentValidatorSetID)
	if err != nil {
		return fmt.Errorf("initialize polkadot listener: %w", err)
	}

	err = relay.ethereumWriter.Start(ctx, eg, requests)
	if err != nil {
		return fmt.Errorf("start ethereum writer: %w", err)
	}

	return nil
}

func (relay *Relay) OneShotSync(ctx context.Context, blockNumber uint64) error {
	// Initialize relaychainConn
	err := relay.relaychainConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	// Initialize ethereumConn
	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
	}
	err = relay.ethereumWriter.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize EthereumWriter: %w", err)
	}

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

func (relay *Relay) RateLimitedSync(ctx context.Context) error {
	var parachainConn *parachain.Connection

	// Initialize parachainConn
	parachainConn = parachain.NewConnection(relay.config.Source.BridgeHubEndpoint, nil)
	err := parachainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return fmt.Errorf("create parachain connection: %w", err)
	}

	// Initialize relaychainConn
	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	// Initialize ethereumConn
	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
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

	for {
		paraNonce, ethNonce, err := relay.queryNonces(ctx, parachainConn, gatewayContract)
		if err != nil {
			return fmt.Errorf("require sync: %w", err)
		}

		if paraNonce > ethNonce {
			beefyBlockHash, err := relay.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
			if err != nil {
				return fmt.Errorf("fetch latest beefy block: %w", err)
			}

			header, err := relay.relaychainConn.API().RPC.Chain.GetHeader(beefyBlockHash)
			if err != nil {
				return fmt.Errorf("fetch latest beefy block: %w", err)
			}

			relay.doSync(ctx, uint64(header.Number))
		}

		// Sleep for 5 minute
		select {
		case <-ctx.Done():
			return nil
		case <-time.After(time.Second * 300):
		}
	}
}

func (relay *Relay) queryNonces(ctx context.Context, parachainConn *parachain.Connection, gatewayContract *contracts.Gateway) (uint64, uint64, error) {
	data, err := types.HexDecodeString("0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539")
	if err != nil {
		return 0, 0, fmt.Errorf("hex decode assethub channel: %w", err)
	}

	assetHubChannelID := *(*[32]byte)(data)

	paraNonce, err := relay.fetchLatestParachainNonce(ctx, assetHubChannelID, parachainConn)
	if err != nil {
		return 0, 0, fmt.Errorf("fetch latest parachain nonce: %w", err)
	}

	ethNonce, err := relay.fetchEthereumNonce(ctx, assetHubChannelID, gatewayContract)
	if err != nil {
		return 0, 0, fmt.Errorf("fetch latest ethereum nonce: %w", err)
	}

	return paraNonce, ethNonce, nil
}

func (r *Relay) fetchLatestParachainNonce(_ context.Context, channelId [32]byte, parachainConn *parachain.Connection) (uint64, error) {
	paraNonceKey, err := types.CreateStorageKey(parachainConn.Metadata(), "EthereumOutboundQueue", "Nonce", channelId[:], nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key for EthereumOutboundQueue.Nonce(%v): %w",
			channelId, err)
	}
	var paraOutboundNonce uint64
	ok, err := parachainConn.API().RPC.State.GetStorageLatest(paraNonceKey, &paraOutboundNonce)
	if err != nil {
		return 0, fmt.Errorf("fetch storage EthereumOutboundQueue.Nonce(%v): %w",
			channelId, err)
	}
	if !ok {
		paraOutboundNonce = 0
	}

	return paraOutboundNonce, nil
}

func (r *Relay) fetchEthereumNonce(ctx context.Context, channelId [32]byte, gatewayContract *contracts.Gateway) (uint64, error) {
	opts := bind.CallOpts{
		Context: ctx,
	}
	ethInboundNonce, _, err := gatewayContract.ChannelNoncesOf(&opts, channelId)
	if err != nil {
		return 0, fmt.Errorf("fetch Gateway.ChannelNoncesOf(%v): %w", channelId, err)
	}

	return ethInboundNonce, nil
}

func (relay *Relay) doSync(ctx context.Context, blockNumber uint64) error {
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
