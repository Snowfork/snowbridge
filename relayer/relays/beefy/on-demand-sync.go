package beefy

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

type OnDemandRelay struct {
	config           *Config
	ethereumConn     *ethereum.Connection
	relaychainConn   *relaychain.Connection
	polkadotListener *PolkadotListener
	ethereumWriter   *EthereumWriter
}

func NewOnDemandRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*OnDemandRelay, error) {
	ethereumConn := ethereum.NewConnection(&config.Sink.Ethereum, ethereumKeypair)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.ParseEndpoints(), nil)

	polkadotListener := NewPolkadotListener(&config.Source, relaychainConn)
	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn)

	return &OnDemandRelay{
		config:           config,
		ethereumConn:     ethereumConn,
		relaychainConn:   relaychainConn,
		polkadotListener: polkadotListener,
		ethereumWriter:   ethereumWriter,
	}, nil
}

// SyncInTwoPhase performs the standard two-phase BEEFY sync flow for a relaychain block.
func (relay *OnDemandRelay) SyncInTwoPhase(ctx context.Context, blockNumber uint64) error {
	task, err := relay.polkadotListener.generateBeefyUpdate(blockNumber)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}
	err = relay.SyncRequestInTwoPhase(ctx, &task)
	if err != nil {
		return fmt.Errorf("Sync beefy request failed: %w", err)
	}
	return nil
}

// SyncRequestInTwoPhase performs the standard two-phase BEEFY sync flow for an already prepared request.
func (relay *OnDemandRelay) SyncRequestInTwoPhase(ctx context.Context, task *Request) error {
	if task == nil {
		return nil
	}

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
	if uint64(task.SignedCommitment.Commitment.BlockNumber) <= state.LatestBeefyBlock {
		logger.Info("Commitment outdated, just ignore")
		return nil
	}

	if task.SignedCommitment.Commitment.ValidatorSetID > state.NextValidatorSetID {
		logger.Warn("Task unexpected, wait for mandatory updates to catch up first")
		return nil
	}

	if task.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		task.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		task.ValidatorsRoot = state.NextValidatorSetRoot
	}
	err = relay.ethereumWriter.submit(ctx, task)
	if err != nil {
		return fmt.Errorf("Sync beefy request failed: %w", err)
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

func (relay *OnDemandRelay) syncFiatShamir(ctx context.Context, blockNumber uint64) error {
	task, err := relay.polkadotListener.generateBeefyUpdate(blockNumber)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}
	err = relay.syncBeefyUpdateFiatShamir(ctx, &task)
	if err != nil {
		return fmt.Errorf("Sync beefy request failed: %w", err)
	}
	return nil
}

func (relay *OnDemandRelay) prepareBeefySyncTask(ctx context.Context, task *Request) (*log.Entry, bool, error) {
	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return nil, false, fmt.Errorf("query beefy client state: %w", err)
	}

	logger := log.WithFields(log.Fields{
		"commitmentBlock":       task.SignedCommitment.Commitment.BlockNumber,
		"latestBeefyBlock":      state.LatestBeefyBlock,
		"currentValidatorSetID": state.CurrentValidatorSetID,
		"nextValidatorSetID":    state.NextValidatorSetID,
	})

	if uint64(task.SignedCommitment.Commitment.BlockNumber) <= state.LatestBeefyBlock {
		logger.Info("Commitment outdated, just ignore")
		return logger, true, nil
	}

	if task.SignedCommitment.Commitment.ValidatorSetID > state.NextValidatorSetID {
		logger.Warn("Task unexpected, wait for mandatory updates to catch up first")
		return logger, true, nil
	}

	if task.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		task.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		task.ValidatorsRoot = state.NextValidatorSetRoot
	}

	return logger, false, nil
}

func (relay *OnDemandRelay) logBeefySyncSuccess(ctx context.Context, logger *log.Entry, task *Request, message string) error {
	updatedState, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}

	logger.WithFields(log.Fields{
		"commitmentBlock":           task.SignedCommitment.Commitment.BlockNumber,
		"updatedBeefyBlock":         updatedState.LatestBeefyBlock,
		"updatedValidatorSetID":     updatedState.CurrentValidatorSetID,
		"updatedNextValidatorSetID": updatedState.NextValidatorSetID,
	}).Info(message)

	return nil
}

func (relay *OnDemandRelay) syncBeefyUpdateFiatShamir(ctx context.Context, task *Request) error {
	logger, skip, err := relay.prepareBeefySyncTask(ctx, task)
	if err != nil {
		return err
	}
	if skip {
		return nil
	}

	err = relay.ethereumWriter.submitFiatShamir(ctx, task)
	if err != nil {
		return fmt.Errorf("fail to submit fiat-shamir beefy update: %w", err)
	}

	return relay.logBeefySyncSuccess(ctx, logger, task, "Sync fiat-shamir beefy update success")
}

func (relay *OnDemandRelay) OneShotStart(ctx context.Context, beefyBlockNumber uint64) error {
	eg, ctx := errgroup.WithContext(ctx)
	err := relay.InitializeOnDemandSync(ctx, eg)
	if err != nil {
		return err
	}

	log.Info("Performing fiat-shamir sync")

	err = relay.syncFiatShamir(ctx, beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("Sync failed: %w", err)
	}

	log.Info("Sync completed")
	return nil
}

// QueryBeefyClientState exposes the on-chain BEEFY light client tip for ops tooling.
func (relay *OnDemandRelay) QueryBeefyClientState(ctx context.Context) (*BeefyClientState, error) {
	return relay.ethereumWriter.queryBeefyClientState(ctx)
}

// RelayBeefyTip returns the relay-chain BEEFY finalized block number.
func (relay *OnDemandRelay) RelayBeefyTip(ctx context.Context) (uint64, error) {
	finalizedHash, err := relay.relaychainConn.API().RPC.Beefy.GetFinalizedHead()
	if err != nil {
		return 0, err
	}
	finalizedHeader, err := relay.relaychainConn.API().RPC.Chain.GetHeader(finalizedHash)
	if err != nil {
		return 0, err
	}
	return uint64(finalizedHeader.Number), nil
}

// SyncNextFromLightClient finds the farthest forward-compatible BEEFY commitment
// after the on-chain light client tip and submits it via fiat-shamir (single tx)
// only when it is a validator-set handover. See SyncNextHandoverFromLightClient.
func (relay *OnDemandRelay) SyncNextFromLightClient(ctx context.Context) error {
	return relay.SyncNextHandoverFromLightClient(ctx, false)
}

func (relay *OnDemandRelay) InitializeOnDemandSync(ctx context.Context, eg *errgroup.Group) error {
	err := relay.ethereumConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Sink.Ethereum.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to ethereum: %w", err)
	}
	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.Polkadot.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("connect to relaychain: %w", err)
	}
	err = relay.ethereumWriter.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize EthereumWriter: %w", err)
	}

	return nil
}

// RotateEthereumEndpoint walks to the next Sink Ethereum RPC after a
// transient error and rebinds BeefyClient/Multicall3. No-op when only one
// endpoint is configured.
func (relay *OnDemandRelay) RotateEthereumEndpoint(ctx context.Context) {
	if relay == nil || relay.ethereumConn == nil {
		return
	}
	if err := relay.ethereumConn.Reconnect(ctx); err != nil {
		log.WithError(err).Error("On-demand Ethereum reconnect after transient error failed")
		return
	}
	if err := relay.ethereumWriter.initialize(ctx); err != nil {
		log.WithError(err).Error("On-demand Ethereum rebind after reconnect failed")
	}
}

type ConsensusTaskResult struct {
	Task                          *Request
	Calldata                      []byte
	BeefyTarget                   common.Address
	MMRAnchorRelayBlock           uint64
	IsHandover                    bool
	LeafNextValidatorSetID        uint64
	LightClientNextValidatorSetID uint64
}

func (relay *OnDemandRelay) BuildConsensusTask(ctx context.Context, blockNumber uint64) (*ConsensusTaskResult, error) {
	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return nil, fmt.Errorf("query beefy client state: %w", err)
	}

	task, err := relay.polkadotListener.generateBeefyUpdateWithMaxValidatorSet(state, blockNumber)
	if err != nil {
		return nil, fmt.Errorf("fail to generate compatible beefy request: %w", err)
	}
	if task == nil {
		return nil, nil
	}

	leafNextValidatorSetID := uint64(task.Proof.Leaf.BeefyNextAuthoritySet.ID)
	lightClientNextValidatorSetID := state.NextValidatorSetID
	isHandover := leafNextValidatorSetID > lightClientNextValidatorSetID
	commitmentBlock := uint64(task.SignedCommitment.Commitment.BlockNumber)

	if task.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		task.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		task.ValidatorsRoot = state.NextValidatorSetRoot
	}

	return &ConsensusTaskResult{
		Task:                          task,
		BeefyTarget:                   relay.ethereumWriter.ContractAddress(),
		MMRAnchorRelayBlock:           commitmentBlock,
		IsHandover:                    isHandover,
		LeafNextValidatorSetID:        leafNextValidatorSetID,
		LightClientNextValidatorSetID: lightClientNextValidatorSetID,
	}, nil
}

// BuildFiatShamirCalldata returns calldata for submitFiatShamir, the BeefyClient address,
// mmrAnchorRelayBlock, and handover metadata.
// mmrAnchorRelayBlock is SignedCommitment.Commitment.BlockNumber: the relay block whose MMR root the submission
// updates on Ethereum. Parachain v2_submit proofs must use this relay block (not the caller's hint block), because
// generateBeefyUpdate may wait for a newer finalized head than blockNumber while the commitment still references
// an earlier relay block.
func (relay *OnDemandRelay) BuildFiatShamirCalldata(ctx context.Context, blockNumber uint64) (*ConsensusTaskResult, error) {
	result, err := relay.BuildConsensusTask(ctx, blockNumber)
	if err != nil {
		return nil, err
	}
	if result == nil {
		return nil, nil
	}

	return relay.BuildFiatShamirCalldataForTask(ctx, result)
}

func (relay *OnDemandRelay) BuildFiatShamirCalldataForTask(ctx context.Context, result *ConsensusTaskResult) (*ConsensusTaskResult, error) {
	if result == nil || result.Task == nil {
		return nil, nil
	}

	calldata, err := relay.ethereumWriter.BuildFiatShamirCalldata(ctx, result.Task)
	if err != nil {
		return nil, fmt.Errorf("build submitFiatShamir calldata: %w", err)
	}

	result.Calldata = calldata
	return result, nil
}

// ExecuteTwoPhaseAndPackSubmitFinal runs the BEEFY two-phase flow then returns
// packed submitFinal calldata for batching with message calls.
func (relay *OnDemandRelay) ExecuteTwoPhaseAndPackSubmitFinal(ctx context.Context, result *ConsensusTaskResult) ([]byte, error) {
	if result == nil || result.Task == nil {
		return nil, nil
	}

	calldata, err := relay.ethereumWriter.ExecuteTwoPhaseAndPackSubmitFinal(ctx, result.Task)
	if err != nil {
		return nil, fmt.Errorf("execute two-phase and pack submitFinal: %w", err)
	}

	return calldata, nil
}

func (relay *OnDemandRelay) BeefyClientAddress() common.Address {
	return relay.ethereumWriter.ContractAddress()
}

func (relay *OnDemandRelay) GetConfig() *Config {
	return relay.config
}

// SetSubmitMutex shares the parachain-v3 account-nonce serializer with BEEFY
// submitInitial / commitPrevRandao broadcasts.
func (relay *OnDemandRelay) SetSubmitMutex(mu *sync.Mutex) {
	if relay.ethereumWriter != nil {
		relay.ethereumWriter.SetSubmitMutex(mu)
	}
}
