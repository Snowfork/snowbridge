package beefy

import (
	"context"
	"encoding/hex"
	"fmt"
	"math/big"
	"math/rand"
	"strings"
	"sync"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/bitfield"

	log "github.com/sirupsen/logrus"
)

type EthereumWriter struct {
	config          *SinkConfig
	conn            *ethereum.Connection
	contract        *contracts.BeefyClient
	multicall3      *contracts.Multicall3
	contractABI     abi.ABI
	blockWaitPeriod uint64
	// submitMu serializes nonce allocation + broadcast when set (shared with
	// parachain-v3 fast-path submits). Receipt waits must run outside the lock.
	submitMu *sync.Mutex
}

type PreparedFinalSubmission struct {
	Params    *FinalRequestParams
	LogFields logrus.Fields
}

func NewEthereumWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
) *EthereumWriter {
	contractABI, err := abi.JSON(strings.NewReader(contracts.BeefyClientABI))
	if err != nil {
		panic(fmt.Sprintf("parse beefy client ABI: %v", err))
	}
	return &EthereumWriter{
		config:      config,
		conn:        conn,
		contractABI: contractABI,
	}
}

// SetSubmitMutex shares a nonce-safe broadcast lock with parachain-v3. Receipt
// waits must run outside the lock so instant/direct-relay can interleave.
func (wr *EthereumWriter) SetSubmitMutex(mu *sync.Mutex) {
	wr.submitMu = mu
}

func (wr *EthereumWriter) withSubmitLock(fn func() (*types.Transaction, error)) (*types.Transaction, error) {
	if wr.submitMu != nil {
		wr.submitMu.Lock()
		defer wr.submitMu.Unlock()
	}
	return fn()
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

	latestBeefyBlock, err := wr.contract.LatestBeefyBlock(&callOpts)
	if err != nil {
		return nil, err
	}

	currentValidatorSet, err := wr.contract.CurrentValidatorSet(&callOpts)
	if err != nil {
		return nil, err
	}
	nextValidatorSet, err := wr.contract.NextValidatorSet(&callOpts)
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
	prepared, err := wr.executeTwoPhaseAndPrepareSubmitFinalDirect(ctx, task)
	if err != nil {
		return err
	}
	if prepared == nil {
		return nil
	}

	tx, err := wr.submitFinal(ctx, prepared, false)
	if err != nil {
		if isDuplicateBeefyError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Commitment already submitted by another relayer during submitFinal, skipping")
			return nil
		}
		return fmt.Errorf("Failed to call submitFinal: %w", err)
	}

	_, err = wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		return fmt.Errorf("Failed to get receipt of submitFinal: %w", err)
	}

	log.WithFields(logrus.Fields{
		"tx":          tx.Hash().Hex(),
		"blockNumber": task.SignedCommitment.Commitment.BlockNumber,
	}).Debug("Transaction SubmitFinal succeeded")

	return nil
}

func (wr *EthereumWriter) executeTwoPhaseAndPrepareSubmitFinalDirect(ctx context.Context, task *Request) (*PreparedFinalSubmission, error) {
	return wr.executeTwoPhaseAndPrepareFinalSubmission(ctx, task, false)
}

func (wr *EthereumWriter) executeTwoPhaseAndPrepareSubmitFinalWithMulticall(ctx context.Context, task *Request) (*PreparedFinalSubmission, error) {
	return wr.executeTwoPhaseAndPrepareFinalSubmission(ctx, task, true)
}

func (wr *EthereumWriter) executeTwoPhaseAndPrepareFinalSubmission(ctx context.Context, task *Request, useMulticall bool) (*PreparedFinalSubmission, error) {
	// Initial submission
	tx, initialBitfield, err := wr.doSubmitInitial(ctx, task, useMulticall)
	if err != nil {
		if isDuplicateBeefyError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Commitment already submitted by another relayer during submitInitial, skipping")
			return nil, nil
		}
		return nil, fmt.Errorf("Failed to call submitInitial: %w", err)
	}
	// Wait for receipt of submitInitial
	receipt, err := wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		return nil, fmt.Errorf("Failed to get receipt of submitInitial: %w", err)
	}
	log.WithFields(logrus.Fields{
		"tx":      tx.Hash().Hex(),
		"receipt": receipt.BlockNumber,
	}).Debug("Transaction submitInitial succeeded")

	// Contract allows commitPrevRandao once:
	//   block.number >= ticket.blockNumber + randaoCommitDelay
	// WaitForFutureBlock returns when latest-submitInitialBlock >= gap, so gap must be
	// randaoCommitDelay (not +1). The old +1 waited one extra block (~12s) past the
	// earliest legal commit. See BeefyClient.commitPrevRandao WaitPeriodNotOver.
	// Background: https://eth2book.info/altair/part3/config/preset/#max_seed_lookahead
	log.Debug(fmt.Sprintf("Waiting RandaoCommitDelay by %d blocks", wr.blockWaitPeriod))
	err = wr.conn.WaitForFutureBlock(ctx, receipt.BlockNumber.Uint64(), wr.blockWaitPeriod)
	if err != nil {
		return nil, fmt.Errorf("Failed to wait for RandaoCommitDelay: %w", err)
	}

	commitmentHash, err := task.CommitmentHash()
	if err != nil {
		return nil, fmt.Errorf("generate commitment hash: %w", err)
	}

	if task.Skippable {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("CommitPrevRandao is skipped, indicating that a newer update is already in progress.")
		return nil, nil
	}
	isTaskOutdated, err := wr.isTaskOutdated(ctx, task)
	if err != nil {
		return nil, fmt.Errorf("check if task is outdated: %w", err)
	}
	if isTaskOutdated {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment already synced")
		return nil, nil
	}
	// Commit PrevRandao which will be used as seed to randomly select subset of validators
	// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/contracts/contracts/BeefyClient.sol#L446-L447
	state, err := wr.queryBeefyClientState(ctx)
	if err != nil {
		return nil, fmt.Errorf("query beefy client state: %w", err)
	}

	// Ignore beefy block already synced
	if uint64(task.SignedCommitment.Commitment.BlockNumber) <= state.LatestBeefyBlock {
		log.WithFields(log.Fields{
			"validatorSetID": state.CurrentValidatorSetID,
			"beefyBlock":     state.LatestBeefyBlock,
			"relayBlock":     task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Beefy block already synced, just ignore")
		return nil, nil
	}
	tx, err = wr.submitCommitPrevRandao(ctx, *commitmentHash, useMulticall)
	if err != nil {
		if isDuplicateBeefyError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Commitment already submitted by another relayer during CommitPrevRandao, skipping")
			return nil, nil
		}
		return nil, fmt.Errorf("Failed to call CommitPrevRandao: %w", err)
	}

	_, err = wr.conn.WatchTransaction(ctx, tx, 0)
	if err != nil {
		return nil, fmt.Errorf("Failed to get receipt of CommitPrevRandao: %w", err)
	}

	if task.Skippable {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("SubmitFinal is skipped, indicating that a newer update is already in progress.")
		return nil, nil
	}
	isTaskOutdated, err = wr.isTaskOutdated(ctx, task)
	if err != nil {
		return nil, fmt.Errorf("check if task is outdated: %w", err)
	}
	if isTaskOutdated {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment already synced")
		return nil, nil
	}
	// Final submission
	state, err = wr.queryBeefyClientState(ctx)
	if err != nil {
		return nil, fmt.Errorf("query beefy client state: %w", err)
	}

	// Ignore beefy block already synced
	if uint64(task.SignedCommitment.Commitment.BlockNumber) <= state.LatestBeefyBlock {
		log.WithFields(log.Fields{
			"validatorSetID": state.CurrentValidatorSetID,
			"beefyBlock":     state.LatestBeefyBlock,
			"relayBlock":     task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Beefy block already synced, just ignore")
		return nil, nil
	}

	prepared, err := wr.prepareSubmitFinal(task, *commitmentHash, initialBitfield, wr.ticketOwnerAddress(useMulticall))
	if err != nil {
		return nil, fmt.Errorf("prepare submitFinal: %w", err)
	}

	return prepared, nil
}

func (wr *EthereumWriter) doSubmitInitial(ctx context.Context, task *Request, useMulticall bool) (*types.Transaction, []*big.Int, error) {
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

	initialBitfield, err := wr.contract.CreateInitialBitfield(
		&bind.CallOpts{
			Pending: true,
			From:    wr.conn.Keypair().CommonAddress(),
		},
		signedValidators, validatorCount,
	)
	if err != nil {
		return nil, nil, fmt.Errorf("create initial bitfield: %w", err)
	}

	msg, err := task.MakeSubmitInitialParams(chosenValidator, initialBitfield)
	if err != nil {
		return nil, nil, err
	}

	var tx *types.Transaction
	tx, err = wr.submitInitial(ctx, msg, useMulticall)
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
func (wr *EthereumWriter) prepareSubmitFinal(task *Request, commitmentHash [32]byte, initialBitfield []*big.Int, ticketOwner common.Address) (*PreparedFinalSubmission, error) {
	finalBitfield, err := wr.contract.CreateFinalBitfield(
		&bind.CallOpts{
			Pending: true,
			From:    ticketOwner,
		},
		commitmentHash,
		initialBitfield,
	)

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

	return &PreparedFinalSubmission{
		Params:    params,
		LogFields: logFields,
	}, nil
}

func (wr *EthereumWriter) submitFinal(ctx context.Context, prepared *PreparedFinalSubmission, useMulticall bool) (*types.Transaction, error) {
	if useMulticall && wr.multicall3 != nil {
		calldata, err := wr.contractABI.Pack(
			"submitFinal",
			prepared.Params.Commitment,
			prepared.Params.Bitfield,
			prepared.Params.Proofs,
			prepared.Params.Leaf,
			prepared.Params.LeafProof,
			prepared.Params.LeafProofOrder,
		)
		if err != nil {
			return nil, fmt.Errorf("pack submitFinal calldata: %w", err)
		}

		tx, err := wr.submitBeefyCallViaMulticall3(ctx, calldata, "SubmitFinal", prepared.LogFields)
		if err != nil {
			return nil, fmt.Errorf("final submission via Multicall3: %w", err)
		}

		return tx, nil
	}

	tx, err := wr.withSubmitLock(func() (*types.Transaction, error) {
		return wr.contract.SubmitFinal(
			wr.conn.MakeTxOpts(ctx),
			prepared.Params.Commitment,
			prepared.Params.Bitfield,
			prepared.Params.Proofs,
			prepared.Params.Leaf,
			prepared.Params.LeafProof,
			prepared.Params.LeafProofOrder,
		)
	})
	if err != nil {
		return nil, fmt.Errorf("final submission: %w", err)
	}

	log.WithField("txHash", tx.Hash().Hex()).
		WithFields(prepared.LogFields).
		Info("Sent SubmitFinal transaction")

	return tx, nil
}

func (wr *EthereumWriter) initialize(ctx context.Context) error {
	address := common.HexToAddress(wr.config.Contracts.BeefyClient)
	contract, err := contracts.NewBeefyClient(address, wr.conn.Client())
	if err != nil {
		return fmt.Errorf("create beefy client: %w", err)
	}
	wr.contract = contract
	if wr.config.Contracts.Multicall3 != "" {
		multicall3, err := contracts.NewMulticall3(common.HexToAddress(wr.config.Contracts.Multicall3), wr.conn.Client())
		if err != nil {
			return fmt.Errorf("create multicall3: %w", err)
		}
		wr.multicall3 = multicall3
	}

	callOpts := bind.CallOpts{
		Context: ctx,
	}
	blockWaitPeriod, err := wr.contract.RandaoCommitDelay(&callOpts)
	if err != nil {
		return fmt.Errorf("create randao commit delay: %w", err)
	}
	wr.blockWaitPeriod = blockWaitPeriod.Uint64()
	log.WithField("randaoCommitDelay", wr.blockWaitPeriod).Trace("Fetched randaoCommitDelay")

	return nil
}

func (wr *EthereumWriter) submitFiatShamir(ctx context.Context, task *Request) error {
	params, logFields, err := wr.prepareFiatShamirParams(ctx, task)
	if err != nil {
		return fmt.Errorf("prepare Fiat-Shamir params: %w", err)
	}

	// Check if task is outdated before final submission
	isTaskOutdated, err := wr.isTaskOutdated(ctx, task)
	if err != nil {
		return fmt.Errorf("check if task is outdated: %w", err)
	}
	if isTaskOutdated {
		log.WithFields(logrus.Fields{
			"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment already synced, skipping SubmitFiatShamir")
		return nil
	}

	tx, err := wr.contract.SubmitFiatShamir(
		wr.conn.MakeTxOpts(ctx),
		params.Commitment,
		params.Bitfield,
		params.Proofs,
		params.Leaf,
		params.LeafProof,
		params.LeafProofOrder,
	)
	if err != nil {
		// Check if error is due to commitment already being submitted (duplicate)
		if isDuplicateBeefyError(err) {
			log.WithFields(logrus.Fields{
				"beefyBlock": task.SignedCommitment.Commitment.BlockNumber,
			}).Info("Commitment was already submitted by another relayer, skipping")
			return nil
		}
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

func (wr *EthereumWriter) prepareFiatShamirParams(ctx context.Context, task *Request) (*FinalRequestParams, logrus.Fields, error) {
	signedValidators := []*big.Int{}
	for i, signature := range task.SignedCommitment.Signatures {
		if signature.IsSome() {
			signedValidators = append(signedValidators, big.NewInt(int64(i)))
		}
	}
	validatorCount := big.NewInt(int64(len(task.SignedCommitment.Signatures)))

	initialBitfield, err := wr.contract.CreateInitialBitfield(
		&bind.CallOpts{
			Pending: true,
			From:    wr.conn.Keypair().CommonAddress(),
		},
		signedValidators, validatorCount,
	)
	if err != nil {
		return nil, nil, fmt.Errorf("create initial bitfield: %w", err)
	}

	commitment := toBeefyClientCommitment(&task.SignedCommitment.Commitment)

	finalBitfield, err := wr.contract.CreateFiatShamirFinalBitfield(
		&bind.CallOpts{
			Pending: true,
			From:    wr.conn.Keypair().CommonAddress(),
		},
		*commitment,
		initialBitfield,
	)

	if err != nil {
		return nil, nil, fmt.Errorf("create validator final bitfield: %w", err)
	}

	validatorIndices := bitfield.New(finalBitfield).Members()

	params, err := task.MakeSubmitFinalParams(validatorIndices, initialBitfield)
	if err != nil {
		return nil, nil, fmt.Errorf("make submit final params: %w", err)
	}

	logFields, err := wr.makeSubmitFinalLogFields(task, params)
	if err != nil {
		return nil, nil, fmt.Errorf("logging params: %w", err)
	}

	return params, logFields, nil
}

func (wr *EthereumWriter) ticketOwnerAddress(useMulticall bool) common.Address {
	if useMulticall && wr.multicall3 != nil {
		return common.HexToAddress(wr.config.Contracts.Multicall3)
	}

	return wr.conn.Keypair().CommonAddress()
}

func (wr *EthereumWriter) submitInitial(ctx context.Context, msg *InitialRequestParams, useMulticall bool) (*types.Transaction, error) {
	if !useMulticall || wr.multicall3 == nil {
		return wr.withSubmitLock(func() (*types.Transaction, error) {
			return wr.contract.SubmitInitial(
				wr.conn.MakeTxOpts(ctx),
				msg.Commitment,
				msg.Bitfield,
				msg.Proof,
			)
		})
	}

	calldata, err := wr.contractABI.Pack("submitInitial", msg.Commitment, msg.Bitfield, msg.Proof)
	if err != nil {
		return nil, fmt.Errorf("pack submitInitial calldata: %w", err)
	}

	return wr.submitBeefyCallViaMulticall3(ctx, calldata, "SubmitInitial", nil)
}

func (wr *EthereumWriter) submitCommitPrevRandao(ctx context.Context, commitmentHash [32]byte, useMulticall bool) (*types.Transaction, error) {
	if !useMulticall || wr.multicall3 == nil {
		return wr.withSubmitLock(func() (*types.Transaction, error) {
			return wr.contract.CommitPrevRandao(
				wr.conn.MakeTxOpts(ctx),
				commitmentHash,
			)
		})
	}

	calldata, err := wr.contractABI.Pack("commitPrevRandao", commitmentHash)
	if err != nil {
		return nil, fmt.Errorf("pack commitPrevRandao calldata: %w", err)
	}

	return wr.submitBeefyCallViaMulticall3(ctx, calldata, "CommitPrevRandao", logrus.Fields{
		"commitmentHash": "0x" + hex.EncodeToString(commitmentHash[:]),
	})
}

func (wr *EthereumWriter) submitBeefyCallViaMulticall3(ctx context.Context, calldata []byte, step string, fields logrus.Fields) (*types.Transaction, error) {
	if wr.multicall3 == nil {
		return nil, fmt.Errorf("multicall3 is not configured")
	}

	tx, err := wr.withSubmitLock(func() (*types.Transaction, error) {
		return wr.multicall3.Aggregate3(wr.conn.MakeTxOpts(ctx), []contracts.Multicall3Call3{{
			Target:       wr.ContractAddress(),
			AllowFailure: false,
			CallData:     calldata,
		}})
	})
	if err != nil {
		return nil, err
	}

	logFields := logrus.Fields{
		"txHash":             tx.Hash().Hex(),
		"multicall3Contract": common.HexToAddress(wr.config.Contracts.Multicall3).Hex(),
		"beefyClient":        wr.ContractAddress().Hex(),
	}
	for key, value := range fields {
		logFields[key] = value
	}

	log.WithFields(logFields).Infof("Sent %s transaction via Multicall3", step)

	return tx, nil
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

// isDuplicateBeefyError checks if the error indicates the commitment was already submitted
func isDuplicateBeefyError(err error) bool {
	if err == nil {
		return false
	}
	errStr := err.Error()
	// Check for common duplicate/already processed error patterns from the BeefyClient contract
	return strings.Contains(errStr, "StaleCommitment") ||
		strings.Contains(errStr, "InvalidCommitment") ||
		strings.Contains(errStr, "already") ||
		strings.Contains(errStr, "Duplicate")
}

func (wr *EthereumWriter) ContractAddress() common.Address {
	return common.HexToAddress(wr.config.Contracts.BeefyClient)
}

func (wr *EthereumWriter) BuildFiatShamirCalldata(ctx context.Context, task *Request) ([]byte, error) {
	params, _, err := wr.prepareFiatShamirParams(ctx, task)
	if err != nil {
		return nil, err
	}

	calldata, err := wr.contractABI.Pack(
		"submitFiatShamir",
		params.Commitment,
		params.Bitfield,
		params.Proofs,
		params.Leaf,
		params.LeafProof,
		params.LeafProofOrder,
	)
	if err != nil {
		return nil, fmt.Errorf("pack submitFiatShamir calldata: %w", err)
	}

	return calldata, nil
}

// ExecuteTwoPhaseAndPackSubmitFinal runs submitInitial, waits for Randao delay,
// commits prevRandao, then packs submitFinal calldata for the caller to batch
// (e.g. Multicall3 with message delivers). Returns nil calldata when the
// commitment was already synced/skipped.
func (wr *EthereumWriter) ExecuteTwoPhaseAndPackSubmitFinal(ctx context.Context, task *Request) ([]byte, error) {
	prepared, err := wr.executeTwoPhaseAndPrepareSubmitFinalWithMulticall(ctx, task)
	if err != nil {
		return nil, err
	}
	if prepared == nil {
		return nil, nil
	}

	calldata, err := wr.contractABI.Pack(
		"submitFinal",
		prepared.Params.Commitment,
		prepared.Params.Bitfield,
		prepared.Params.Proofs,
		prepared.Params.Leaf,
		prepared.Params.LeafProof,
		prepared.Params.LeafProofOrder,
	)
	if err != nil {
		return nil, fmt.Errorf("pack submitFinal calldata: %w", err)
	}

	log.WithFields(prepared.LogFields).Info("Prepared SubmitFinal calldata after submitInitial and CommitPrevRandao")

	return calldata, nil
}
