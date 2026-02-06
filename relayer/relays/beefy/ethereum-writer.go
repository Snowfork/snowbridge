package beefy

import (
	"context"
	"encoding/hex"
	"fmt"
	"math/big"
	"math/rand"
	"strings"
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

// Expected error selectors that should not cause relayer restarts
const (
	// TicketAlreadyOwned: Another relayer already claimed this commitment (from BeefyClientWrapper)
	ErrTicketAlreadyOwned = "0x60bbe44e"
	// StaleCommitment: The commitment has already been synced (from BeefyClient)
	ErrStaleCommitment = "0x3d618e50"
	// NotTicketOwner: Caller is not the ticket owner (from BeefyClientWrapper)
	ErrNotTicketOwner = "0xe18d39ad"
	// InvalidCommitment: The commitment's validator set doesn't match current or next (from BeefyClient)
	ErrInvalidCommitment = "0xc06789fa"
)

// SessionTimeout is the duration after which a pending session is considered expired
const SessionTimeout = 40 * time.Minute

// JsonError interface for extracting error data from Ethereum RPC errors
type JsonError interface {
	Error() string
	ErrorCode() int
	ErrorData() interface{}
}

// isExpectedCompetitionError checks if an error is due to normal relayer competition
// (e.g., another relayer already claimed the commitment or it was already synced)
func isExpectedCompetitionError(err error) bool {
	if err == nil {
		return false
	}

	// First check if the error string contains the hex codes (for wrapped errors)
	errStr := err.Error()
	if strings.Contains(errStr, ErrTicketAlreadyOwned) ||
		strings.Contains(errStr, ErrStaleCommitment) ||
		strings.Contains(errStr, ErrNotTicketOwner) ||
		strings.Contains(errStr, ErrInvalidCommitment) {
		return true
	}

	// Try to extract error data from JsonError interface
	var currentErr error = err
	for currentErr != nil {
		if jsonErr, ok := currentErr.(JsonError); ok {
			errorData := fmt.Sprintf("%v", jsonErr.ErrorData())
			if strings.Contains(errorData, ErrTicketAlreadyOwned) ||
				strings.Contains(errorData, ErrStaleCommitment) ||
				strings.Contains(errorData, ErrNotTicketOwner) ||
				strings.Contains(errorData, ErrInvalidCommitment) {
				return true
			}
		}
		// Try to unwrap
		if unwrapper, ok := currentErr.(interface{ Unwrap() error }); ok {
			currentErr = unwrapper.Unwrap()
		} else {
			break
		}
	}

	return false
}

type EthereumWriter struct {
	config          *SinkConfig
	conn            *ethereum.Connection
	useWrapper      bool
	wrapperContract *contracts.BeefyClientWrapper
	beefyClient     *contracts.BeefyClient
	blockWaitPeriod uint64
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
			}).Info("Skipping commitment: another relayer won the race or commitment already synced")
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
			}).Info("Skipping commitment: another relayer won the race during CommitPrevRandao")
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
			}).Info("Skipping commitment: another relayer won the race during submitFinal")
			return nil
		}
		return fmt.Errorf("Failed to call submitFinal: %w", err)
	}

	_, err = wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		if isExpectedCompetitionError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Skipping commitment: another relayer won the race during submitFinal")
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

	// Check if the session has expired (> 40 minutes old)
	pendingTimestamp, err := wr.wrapperContract.HighestPendingBlockTimestamp(&callOpts)
	if err != nil {
		return false, fmt.Errorf("get highest pending block timestamp: %w", err)
	}

	sessionAge := time.Now().Unix() - pendingTimestamp.Int64()
	if sessionAge > int64(SessionTimeout.Seconds()) {
		log.WithFields(logrus.Fields{
			"highestPendingBlock": highestPendingBlock.Uint64(),
			"sessionAgeMinutes":   sessionAge / 60,
		}).Info("Pending session has expired, proceeding with new submission")
		return false, nil
	}

	log.WithFields(logrus.Fields{
		"highestPendingBlock": highestPendingBlock.Uint64(),
		"latestBeefyBlock":    latestBeefyBlock,
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

func (wr *EthereumWriter) createFiatShamirFinalBitfield(commitment *contracts.IBeefyClientCommitment, initialBitfield []*big.Int) ([]*big.Int, error) {
	callOpts := &bind.CallOpts{
		Pending: true,
		From:    wr.conn.Keypair().CommonAddress(),
	}
	return wr.beefyClient.CreateFiatShamirFinalBitfield(callOpts, ToBeefyClientCommitment(commitment), initialBitfield)
}

func (wr *EthereumWriter) submitInitial(ctx context.Context, msg *InitialRequestParams) (*types.Transaction, error) {
	if wr.useWrapper {
		return wr.wrapperContract.SubmitInitial(
			wr.conn.MakeTxOpts(ctx),
			msg.Commitment,
			msg.Bitfield,
			msg.Proof,
		)
	}
	return wr.beefyClient.SubmitInitial(
		wr.conn.MakeTxOpts(ctx),
		ToBeefyClientCommitment(&msg.Commitment),
		msg.Bitfield,
		ToBeefyClientValidatorProof(&msg.Proof),
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
		return wr.wrapperContract.SubmitFinal(
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
		ToBeefyClientCommitment(&params.Commitment),
		params.Bitfield,
		ToBeefyClientValidatorProofs(params.Proofs),
		ToBeefyClientMMRLeaf(&params.Leaf),
		params.LeafProof,
		params.LeafProofOrder,
	)
}

func (wr *EthereumWriter) doSubmitFiatShamir(ctx context.Context, params *FinalRequestParams) (*types.Transaction, error) {
	if wr.useWrapper {
		return wr.wrapperContract.SubmitFiatShamir(
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
		ToBeefyClientCommitment(&params.Commitment),
		params.Bitfield,
		ToBeefyClientValidatorProofs(params.Proofs),
		ToBeefyClientMMRLeaf(&params.Leaf),
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
	return wr.beefyClient.ComputeCommitmentHash(callOpts, ToBeefyClientCommitment(commitment))
}
