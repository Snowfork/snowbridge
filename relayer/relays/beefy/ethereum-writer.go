package beefy

import (
	"context"
	"encoding/hex"
	"fmt"
	"math/big"
	"math/rand"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/bitfield"

	log "github.com/sirupsen/logrus"
)

// SessionTimeout is the duration after which a pending session is considered expired.
// This should be longer than a full submission cycle (submitInitial → RandaoCommitDelay →
// commitPrevRandao → submitFinal) to avoid competing with legitimate in-progress sessions.
const SessionTimeout = 40 * time.Minute

type EthereumWriter struct {
	config              *SinkConfig
	conn                *ethereum.Connection
	useWrapper          bool
	wrapperContract     *contracts.BeefyClientWrapper
	wrapperBeefyClient  *contracts.BeefyClient
	beefyClient         *contracts.BeefyClient
	blockWaitPeriod     uint64
}

func NewEthereumWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
) *EthereumWriter {
	return &EthereumWriter{
		config: config,
		conn:   conn,
	}
}

func (wr *EthereumWriter) Start(ctx context.Context, eg *errgroup.Group, requests <-chan Request) error {
	// launch task processor
	eg.Go(func() error {
		for {
			select {
			case <-ctx.Done():
				return nil
			case task, ok := <-requests:
				if !ok {
					return nil
				}

				state, err := wr.queryBeefyClientState(ctx)
				if err != nil {
					return fmt.Errorf("query beefy client state: %w", err)
				}

				if task.SignedCommitment.Commitment.BlockNumber < uint32(state.LatestBeefyBlock) {
					log.WithFields(logrus.Fields{
						"beefyBlockNumber": task.SignedCommitment.Commitment.BlockNumber,
						"latestBeefyBlock": state.LatestBeefyBlock,
					}).Info("Commitment already synced")
					continue
				}

				// Mandatory commitments are always signed by the next validator set recorded in
				// the beefy light client
				task.ValidatorsRoot = state.NextValidatorSetRoot

				if wr.config.EnableFiatShamir {
					err = wr.submitFiatShamir(ctx, &task)
				} else {
					err = wr.submit(ctx, &task)
				}
				if err != nil {
					return fmt.Errorf("submit request: %w", err)
				}
			}
		}
	})

	return nil
}

type BeefyClientState struct {
	LatestBeefyBlock        uint64
	CurrentValidatorSetID   uint64
	CurrentValidatorSetRoot [32]byte
	NextValidatorSetID      uint64
	NextValidatorSetRoot    [32]byte
}

func (wr *EthereumWriter) queryBeefyClientState(ctx context.Context) (*BeefyClientState, error) {
	callOpts := bind.CallOpts{
		Context: ctx,
	}

	latestBeefyBlock, err := wr.getLatestBeefyBlock(&callOpts)
	if err != nil {
		return nil, err
	}

	currentValidatorSet, err := wr.getCurrentValidatorSet(&callOpts)
	if err != nil {
		return nil, err
	}

	nextValidatorSet, err := wr.getNextValidatorSet(&callOpts)
	if err != nil {
		return nil, err
	}

	return &BeefyClientState{
		LatestBeefyBlock:        latestBeefyBlock,
		CurrentValidatorSetID:   currentValidatorSet.Id.Uint64(),
		CurrentValidatorSetRoot: currentValidatorSet.Root,
		NextValidatorSetID:      nextValidatorSet.Id.Uint64(),
		NextValidatorSetRoot:    nextValidatorSet.Root,
	}, nil
}

func (wr *EthereumWriter) submit(ctx context.Context, task *Request) error {
	// Dynamically disable wrapper if it has insufficient funds
	wr.useWrapper = wr.isWrapperFunded(ctx)

	// Check if another relayer already has a session in progress (wrapper only)
	if wr.useWrapper {
		shouldSkip, err := wr.shouldSkipDueToPendingSession(ctx, task)
		if err != nil {
			return fmt.Errorf("check pending session: %w", err)
		}
		if shouldSkip {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Skipping submission: another session already in progress with sufficient progress")
			return nil
		}
	}

	// Initial submission
	tx, initialBitfield, err := wr.doSubmitInitial(ctx, task)
	if err != nil {
		return fmt.Errorf("Failed to call submitInitial: %w", err)
	}
	// Wait for receipt of submitInitial
	receipt, err := wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		if isExpectedCompetitionError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Skipping commitment: expected error (race condition, stale commitment, or validator set mismatch)")
			return nil
		}
		return fmt.Errorf("Failed to get receipt of submitInitial: %w", err)
	}
	log.WithFields(logrus.Fields{
		"tx":      tx.Hash().Hex(),
		"receipt": receipt.BlockNumber,
	}).Debug("Transaction submitInitial succeeded")

	log.Debug(fmt.Sprintf("Waiting RandaoCommitDelay by %d blocks", wr.blockWaitPeriod+1))

	// Wait RandaoCommitDelay before submit CommitPrevRandao to prevent attacker from manipulating committee memberships
	// Details in https://eth2book.info/altair/part3/config/preset/#max_seed_lookahead
	err = wr.conn.WaitForFutureBlock(ctx, receipt.BlockNumber.Uint64(), wr.blockWaitPeriod+1)
	if err != nil {
		return fmt.Errorf("Failed to wait for RandaoCommitDelay: %w", err)
	}

	commitmentHash, err := wr.computeCommitmentHash(task)
	if err != nil {
		return fmt.Errorf("compute commitment hash: %w", err)
	}

	if task.Skippable {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("CommitPrevRandao is skipped, indicating that a newer update is already in progress.")
		return nil
	}
	isTaskOutdated, err := wr.isTaskOutdated(ctx, task)
	if err != nil {
		return fmt.Errorf("check if task is outdated: %w", err)
	}
	if isTaskOutdated {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment already synced")
		return nil
	}

	// Re-check progress after RandaoCommitDelay (another relayer may have advanced the light client)
	insufficient, err := wr.isProgressInsufficient(ctx, task)
	if err != nil {
		return fmt.Errorf("check progress before commitPrevRandao: %w", err)
	}
	if insufficient {
		return nil
	}

	// Commit PrevRandao which will be used as seed to randomly select subset of validators
	// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/contracts/contracts/BeefyClient.sol#L446-L447
	tx, err = wr.doCommitPrevRandao(ctx, commitmentHash)
	if err != nil {
		return fmt.Errorf("Failed to call CommitPrevRandao: %w", err)
	}

	_, err = wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		if isExpectedCompetitionError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Skipping commitment: expected error during CommitPrevRandao")
			return nil
		}
		return fmt.Errorf("Failed to get receipt of CommitPrevRandao: %w", err)
	}

	if task.Skippable {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("SubmitFinal is skipped, indicating that a newer update is already in progress.")
		return nil
	}
	isTaskOutdated, err = wr.isTaskOutdated(ctx, task)
	if err != nil {
		return fmt.Errorf("check if task is outdated: %w", err)
	}
	if isTaskOutdated {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment already synced")
		return nil
	}

	// Final submission
	tx, err = wr.doSubmitFinal(ctx, commitmentHash, initialBitfield, task)
	if err != nil {
		if isExpectedCompetitionError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Skipping commitment: expected error during submitFinal")
			return nil
		}
		return fmt.Errorf("Failed to call submitFinal: %w", err)
	}

	_, err = wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		if isExpectedCompetitionError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Skipping commitment: expected error during submitFinal receipt")
			return nil
		}
		return fmt.Errorf("Failed to get receipt of submitFinal: %w", err)
	}

	log.WithFields(logrus.Fields{
		"tx":          tx.Hash().Hex(),
		"blockNumber": task.SignedCommitment.Commitment.BlockNumber,
	}).Debug("Transaction SubmitFinal succeeded")

	return nil
}

func (wr *EthereumWriter) doSubmitInitial(ctx context.Context, task *Request) (*types.Transaction, []*big.Int, error) {
	signedValidators := []*big.Int{}
	for i, signature := range task.SignedCommitment.Signatures {
		if signature.IsSome() {
			signedValidators = append(signedValidators, big.NewInt(int64(i)))
		}
	}
	validatorCount := big.NewInt(int64(len(task.SignedCommitment.Signatures)))

	// Pick a random validator who signs beefy commitment
	chosenValidator := signedValidators[rand.Intn(len(signedValidators))].Int64()

	log.WithFields(logrus.Fields{
		"validatorCount":       validatorCount,
		"signedValidators":     signedValidators,
		"signedValidatorCount": len(signedValidators),
		"chosenValidator":      chosenValidator,
	}).Info("Creating initial bitfield")

	initialBitfield, err := wr.createInitialBitfield(signedValidators, validatorCount)
	if err != nil {
		return nil, nil, fmt.Errorf("create initial bitfield: %w", err)
	}

	msg, err := task.MakeSubmitInitialParams(chosenValidator, initialBitfield)
	if err != nil {
		return nil, nil, err
	}

	tx, err := wr.submitInitial(ctx, msg)
	if err != nil {
		return nil, nil, fmt.Errorf("initial submit: %w", err)
	}

	commitmentHash, err := task.CommitmentHash()
	if err != nil {
		return nil, nil, fmt.Errorf("create commitment hash: %w", err)
	}
	log.WithFields(logrus.Fields{
		"txHash":         tx.Hash().Hex(),
		"CommitmentHash": "0x" + hex.EncodeToString(commitmentHash[:]),
		"Commitment":     commitmentToLog(msg.Commitment),
		"Bitfield":       bitfieldToStrings(msg.Bitfield),
		"Proof":          proofToLog(msg.Proof),
	}).Info("Transaction submitted for initial verification")

	return tx, initialBitfield, nil
}

// doSubmitFinal sends a SubmitFinal tx to the BeefyClient contract
func (wr *EthereumWriter) doSubmitFinal(ctx context.Context, commitmentHash [32]byte, initialBitfield []*big.Int, task *Request) (*types.Transaction, error) {
	finalBitfield, err := wr.createFinalBitfield(commitmentHash, initialBitfield)
	if err != nil {
		return nil, fmt.Errorf("create validator bitfield: %w", err)
	}

	validatorIndices := bitfield.New(finalBitfield).Members()

	params, err := task.MakeSubmitFinalParams(validatorIndices, initialBitfield)
	if err != nil {
		return nil, err
	}

	logFields, err := wr.makeSubmitFinalLogFields(task, params)
	if err != nil {
		return nil, fmt.Errorf("logging params: %w", err)
	}

	tx, err := wr.submitFinal(ctx, params)
	if err != nil {
		return nil, fmt.Errorf("final submission: %w", err)
	}

	log.WithField("txHash", tx.Hash().Hex()).
		WithFields(logFields).
		Info("Sent SubmitFinal transaction")

	return tx, nil
}

func (wr *EthereumWriter) initialize(ctx context.Context) error {
	callOpts := bind.CallOpts{
		Context: ctx,
	}

	beefyClientAddress := common.HexToAddress(wr.config.Contracts.BeefyClient)
	beefyClient, err := contracts.NewBeefyClient(beefyClientAddress, wr.conn.Client())
	if err != nil {
		return fmt.Errorf("create beefy client: %w", err)
	}
	wr.beefyClient = beefyClient

	blockWaitPeriod, err := wr.beefyClient.RandaoCommitDelay(&callOpts)
	if err != nil {
		return fmt.Errorf("get randao commit delay: %w", err)
	}
	wr.blockWaitPeriod = blockWaitPeriod.Uint64()

	// Optionally initialize wrapper for state-changing functions with gas refunds
	if wr.config.Contracts.BeefyClientWrapper != "" {
		wr.useWrapper = true
		wrapperAddress := common.HexToAddress(wr.config.Contracts.BeefyClientWrapper)

		wrapperContract, err := contracts.NewBeefyClientWrapper(wrapperAddress, wr.conn.Client())
		if err != nil {
			return fmt.Errorf("create beefy client wrapper: %w", err)
		}
		wr.wrapperContract = wrapperContract

		// BeefyClient binding at the wrapper address for submitInitial/submitFinal/submitFiatShamir
		// (same ABI signatures, routed through wrapper for gas refunds)
		wrapperBeefyClient, err := contracts.NewBeefyClient(wrapperAddress, wr.conn.Client())
		if err != nil {
			return fmt.Errorf("create wrapper beefy client: %w", err)
		}
		wr.wrapperBeefyClient = wrapperBeefyClient

		log.WithFields(logrus.Fields{
			"beefyClient":       beefyClientAddress.Hex(),
			"wrapper":           wrapperAddress.Hex(),
			"randaoCommitDelay": wr.blockWaitPeriod,
		}).Info("Using BeefyClientWrapper for gas refunds")
	} else {
		wr.useWrapper = false
		log.WithFields(logrus.Fields{
			"beefyClient":       beefyClientAddress.Hex(),
			"randaoCommitDelay": wr.blockWaitPeriod,
		}).Info("Using BeefyClient directly (no gas refunds)")
	}

	return nil
}

func (wr *EthereumWriter) isTaskOutdated(ctx context.Context, task *Request) (bool, error) {
	state, err := wr.queryBeefyClientState(ctx)
	if err != nil {
		return false, fmt.Errorf("query beefy client state: %w", err)
	}

	if task.SignedCommitment.Commitment.BlockNumber <= uint32(state.LatestBeefyBlock) {
		return true, nil
	}
	return false, nil
}

func (wr *EthereumWriter) submitFiatShamir(ctx context.Context, task *Request) error {
	// Dynamically disable wrapper if it has insufficient funds
	wr.useWrapper = wr.isWrapperFunded(ctx)

	// Check if expected progress is sufficient for refund (wrapper only)
	insufficient, err := wr.isProgressInsufficient(ctx, task)
	if err != nil {
		return fmt.Errorf("check progress: %w", err)
	}
	if insufficient {
		return nil
	}

	// Check if another relayer already has a session in progress (wrapper only)
	if wr.useWrapper {
		shouldSkip, err := wr.shouldSkipDueToPendingSession(ctx, task)
		if err != nil {
			return fmt.Errorf("check pending session: %w", err)
		}
		if shouldSkip {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Skipping submission: another session already in progress with sufficient progress")
			return nil
		}
	}

	signedValidators := []*big.Int{}
	for i, signature := range task.SignedCommitment.Signatures {
		if signature.IsSome() {
			signedValidators = append(signedValidators, big.NewInt(int64(i)))
		}
	}
	validatorCount := big.NewInt(int64(len(task.SignedCommitment.Signatures)))

	// Pick a random validator who signs beefy commitment
	chosenValidator := signedValidators[rand.Intn(len(signedValidators))].Int64()

	log.WithFields(logrus.Fields{
		"validatorCount":       validatorCount,
		"signedValidators":     signedValidators,
		"signedValidatorCount": len(signedValidators),
		"chosenValidator":      chosenValidator,
	}).Info("Creating initial bitfield")

	initialBitfield, err := wr.createInitialBitfield(signedValidators, validatorCount)
	if err != nil {
		return fmt.Errorf("create initial bitfield: %w", err)
	}

	commitment := toBeefyClientCommitment(&task.SignedCommitment.Commitment)

	finalBitfield, err := wr.createFiatShamirFinalBitfield(commitment, initialBitfield)
	if err != nil {
		return fmt.Errorf("create validator final bitfield: %w", err)
	}

	validatorIndices := bitfield.New(finalBitfield).Members()

	params, err := task.MakeSubmitFinalParams(validatorIndices, initialBitfield)
	if err != nil {
		return fmt.Errorf("make submit final params: %w", err)
	}

	logFields, err := wr.makeSubmitFinalLogFields(task, params)
	if err != nil {
		return fmt.Errorf("logging params: %w", err)
	}

	tx, err := wr.doSubmitFiatShamir(ctx, params)
	if err != nil {
		return fmt.Errorf("SubmitFiatShamir: %w", err)
	}

	log.WithField("txHash", tx.Hash().Hex()).
		WithFields(logFields).
		Info("Sent SubmitFiatShamir transaction")

	_, err = wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		return fmt.Errorf("Wait receipt for SubmitFiatShamir: %w", err)
	}

	log.WithFields(logrus.Fields{
		"tx":          tx.Hash().Hex(),
		"blockNumber": task.SignedCommitment.Commitment.BlockNumber,
	}).Debug("Transaction submitFiatShamir succeeded")

	return nil
}

// isWrapperFunded checks if the wrapper has sufficient balance to cover a refund.
// Returns false if the wrapper is not configured, has insufficient funds, or on error.
func (wr *EthereumWriter) isWrapperFunded(ctx context.Context) bool {
	if !wr.useWrapper {
		return false
	}

	wrapperAddress := common.HexToAddress(wr.config.Contracts.BeefyClientWrapper)
	balance, err := wr.conn.Client().BalanceAt(ctx, wrapperAddress, nil)
	if err != nil {
		log.WithError(err).Warn("Failed to check wrapper balance")
		return false
	}

	callOpts := bind.CallOpts{Context: ctx}
	maxRefund, err := wr.wrapperContract.MaxRefundAmount(&callOpts)
	if err != nil {
		log.WithError(err).Warn("Failed to get maxRefundAmount")
		return false
	}

	if balance.Cmp(maxRefund) < 0 {
		log.WithFields(logrus.Fields{
			"balance":   balance,
			"maxRefund": maxRefund,
		}).Warn("Wrapper has insufficient funds, submitting directly to BeefyClient")
		return false
	}

	return true
}

// isProgressInsufficient checks if the expected progress for this commitment
// would be below the refund target. Only relevant when using the wrapper.
func (wr *EthereumWriter) isProgressInsufficient(ctx context.Context, task *Request) (bool, error) {
	if !wr.useWrapper {
		return false, nil
	}

	callOpts := bind.CallOpts{Context: ctx}

	latestBeefyBlock, err := wr.beefyClient.LatestBeefyBlock(&callOpts)
	if err != nil {
		return false, fmt.Errorf("get latest beefy block: %w", err)
	}

	refundTarget, err := wr.wrapperContract.RefundTarget(&callOpts)
	if err != nil {
		return false, fmt.Errorf("get refund target: %w", err)
	}

	commitmentBlock := uint64(task.SignedCommitment.Commitment.BlockNumber)
	if commitmentBlock <= latestBeefyBlock {
		log.WithFields(logrus.Fields{
			"commitmentBlock":  commitmentBlock,
			"latestBeefyBlock": latestBeefyBlock,
		}).Info("Skipping submission: commitment already behind latest beefy block")
		return true, nil
	}

	progress := commitmentBlock - latestBeefyBlock
	if progress < refundTarget.Uint64() {
		log.WithFields(logrus.Fields{
			"commitmentBlock":  commitmentBlock,
			"latestBeefyBlock": latestBeefyBlock,
			"progress":         progress,
			"refundTarget":     refundTarget.Uint64(),
		}).Info("Skipping submission: expected progress insufficient for refund")
		return true, nil
	}

	return false, nil
}

// shouldSkipDueToPendingSession checks if another relayer already has a session in progress
// that would advance the light client sufficiently. Returns true if we should skip.
// Note: This is only available when using the wrapper contract.
func (wr *EthereumWriter) shouldSkipDueToPendingSession(ctx context.Context, task *Request) (bool, error) {
	callOpts := bind.CallOpts{
		Context: ctx,
	}

	highestPendingBlock, err := wr.wrapperContract.HighestPendingBlock(&callOpts)
	if err != nil {
		return false, fmt.Errorf("get highest pending block: %w", err)
	}

	// No pending session
	if highestPendingBlock.Uint64() == 0 {
		return false, nil
	}

	taskBlock := uint64(task.SignedCommitment.Commitment.BlockNumber)

	// Our commitment is more recent — submit it to advance the light client further
	if taskBlock > highestPendingBlock.Uint64() {
		log.WithFields(logrus.Fields{
			"taskBlock":           taskBlock,
			"highestPendingBlock": highestPendingBlock.Uint64(),
		}).Info("Our commitment is more recent than pending session, proceeding")
		return false, nil
	}

	latestBeefyBlock, err := wr.beefyClient.LatestBeefyBlock(&callOpts)
	if err != nil {
		return false, fmt.Errorf("get latest beefy block: %w", err)
	}

	refundTarget, err := wr.wrapperContract.RefundTarget(&callOpts)
	if err != nil {
		return false, fmt.Errorf("get refund target: %w", err)
	}

	// Check if the pending session would give sufficient progress
	pendingProgress := highestPendingBlock.Uint64() - latestBeefyBlock
	if pendingProgress < refundTarget.Uint64() {
		// Pending session wouldn't give good progress, ok to proceed
		return false, nil
	}

	// Check if the session has expired. SessionTimeout is longer than a full submission
	// cycle to avoid competing with legitimate in-progress sessions.
	pendingTimestamp, err := wr.wrapperContract.HighestPendingBlockTimestamp(&callOpts)
	if err != nil {
		return false, fmt.Errorf("get highest pending block timestamp: %w", err)
	}

	sessionAge := time.Now().Unix() - pendingTimestamp.Int64()
	if sessionAge > int64(SessionTimeout.Seconds()) {
		log.WithFields(logrus.Fields{
			"highestPendingBlock": highestPendingBlock.Uint64(),
			"sessionAgeMinutes":  sessionAge / 60,
		}).Info("Pending session has expired, proceeding with new submission")
		return false, nil
	}

	log.WithFields(logrus.Fields{
		"highestPendingBlock": highestPendingBlock.Uint64(),
		"latestBeefyBlock":    latestBeefyBlock,
		"taskBlock":            taskBlock,
		"pendingProgress":     pendingProgress,
		"sessionAgeMinutes":   sessionAge / 60,
	}).Debug("Active session in progress with sufficient progress")

	return true, nil
}

// Contract abstraction helpers
// View functions always use beefyClient directly
// State-changing functions use wrapper (if configured) or beefyClient

type validatorSetResult struct {
	Id   *big.Int
	Root [32]byte
}

func (wr *EthereumWriter) getLatestBeefyBlock(callOpts *bind.CallOpts) (uint64, error) {
	return wr.beefyClient.LatestBeefyBlock(callOpts)
}

func (wr *EthereumWriter) getCurrentValidatorSet(callOpts *bind.CallOpts) (*validatorSetResult, error) {
	result, err := wr.beefyClient.CurrentValidatorSet(callOpts)
	if err != nil {
		return nil, err
	}
	return &validatorSetResult{Id: result.Id, Root: result.Root}, nil
}

func (wr *EthereumWriter) getNextValidatorSet(callOpts *bind.CallOpts) (*validatorSetResult, error) {
	result, err := wr.beefyClient.NextValidatorSet(callOpts)
	if err != nil {
		return nil, err
	}
	return &validatorSetResult{Id: result.Id, Root: result.Root}, nil
}

func (wr *EthereumWriter) createInitialBitfield(signedValidators []*big.Int, validatorCount *big.Int) ([]*big.Int, error) {
	callOpts := &bind.CallOpts{
		Pending: true,
		From:    wr.conn.Keypair().CommonAddress(),
	}
	return wr.beefyClient.CreateInitialBitfield(callOpts, signedValidators, validatorCount)
}

func (wr *EthereumWriter) createFinalBitfield(commitmentHash [32]byte, initialBitfield []*big.Int) ([]*big.Int, error) {
	// When using wrapper, the ticket was created with msg.sender = wrapper address,
	// so we need to use the wrapper address as From to find the correct ticket
	fromAddr := wr.conn.Keypair().CommonAddress()
	if wr.useWrapper {
		fromAddr = common.HexToAddress(wr.config.Contracts.BeefyClientWrapper)
	}
	callOpts := &bind.CallOpts{
		Pending: true,
		From:    fromAddr,
	}
	return wr.beefyClient.CreateFinalBitfield(callOpts, commitmentHash, initialBitfield)
}

func (wr *EthereumWriter) createFiatShamirFinalBitfield(commitment *contracts.BeefyClientCommitment, initialBitfield []*big.Int) ([]*big.Int, error) {
	callOpts := &bind.CallOpts{
		Pending: true,
		From:    wr.conn.Keypair().CommonAddress(),
	}
	return wr.beefyClient.CreateFiatShamirFinalBitfield(callOpts, *commitment, initialBitfield)
}

func (wr *EthereumWriter) submitInitial(ctx context.Context, msg *InitialRequestParams) (*types.Transaction, error) {
	if wr.useWrapper {
		return wr.wrapperBeefyClient.SubmitInitial(
			wr.conn.MakeTxOpts(ctx),
			msg.Commitment,
			msg.Bitfield,
			msg.Proof,
		)
	}
	return wr.beefyClient.SubmitInitial(
		wr.conn.MakeTxOpts(ctx),
		msg.Commitment,
		msg.Bitfield,
		msg.Proof,
	)
}

func (wr *EthereumWriter) doCommitPrevRandao(ctx context.Context, commitmentHash [32]byte) (*types.Transaction, error) {
	if wr.useWrapper {
		return wr.wrapperContract.CommitPrevRandao(wr.conn.MakeTxOpts(ctx), commitmentHash)
	}
	return wr.beefyClient.CommitPrevRandao(wr.conn.MakeTxOpts(ctx), commitmentHash)
}

func (wr *EthereumWriter) submitFinal(ctx context.Context, params *FinalRequestParams) (*types.Transaction, error) {
	if wr.useWrapper {
		return wr.wrapperBeefyClient.SubmitFinal(
			wr.conn.MakeTxOpts(ctx),
			params.Commitment,
			params.Bitfield,
			params.Proofs,
			params.Leaf,
			params.LeafProof,
			params.LeafProofOrder,
		)
	}
	return wr.beefyClient.SubmitFinal(
		wr.conn.MakeTxOpts(ctx),
		params.Commitment,
		params.Bitfield,
		params.Proofs,
		params.Leaf,
		params.LeafProof,
		params.LeafProofOrder,
	)
}

func (wr *EthereumWriter) doSubmitFiatShamir(ctx context.Context, params *FinalRequestParams) (*types.Transaction, error) {
	if wr.useWrapper {
		return wr.wrapperBeefyClient.SubmitFiatShamir(
			wr.conn.MakeTxOpts(ctx),
			params.Commitment,
			params.Bitfield,
			params.Proofs,
			params.Leaf,
			params.LeafProof,
			params.LeafProofOrder,
		)
	}
	return wr.beefyClient.SubmitFiatShamir(
		wr.conn.MakeTxOpts(ctx),
		params.Commitment,
		params.Bitfield,
		params.Proofs,
		params.Leaf,
		params.LeafProof,
		params.LeafProofOrder,
	)
}

func (wr *EthereumWriter) computeCommitmentHash(task *Request) ([32]byte, error) {
	callOpts := &bind.CallOpts{
		Pending: true,
		From:    wr.conn.Keypair().CommonAddress(),
	}
	commitment := toBeefyClientCommitment(&task.SignedCommitment.Commitment)
	hash, err := wr.beefyClient.ComputeCommitmentHash(callOpts, *commitment)
	if err != nil {
		log.WithError(err).Debug("On-chain computeCommitmentHash not available, falling back to local computation")
		localHash, localErr := task.CommitmentHash()
		if localErr != nil {
			return [32]byte{}, localErr
		}
		return *localHash, nil
	}
	return hash, nil
}
