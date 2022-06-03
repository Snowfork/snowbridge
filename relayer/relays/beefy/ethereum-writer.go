package beefy

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"time"

	"golang.org/x/sync/errgroup"

	goEthereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts/beefyclient"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/bitfield"

	log "github.com/sirupsen/logrus"
)

type EthereumWriter struct {
	config          *SinkConfig
	conn            *ethereum.Connection
	contract        *beefyclient.BeefyClient
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

	address := common.HexToAddress(wr.config.Contracts.BeefyClient)
	contract, err := beefyclient.NewBeefyClient(address, wr.conn.Client())
	if err != nil {
		return err
	}
	wr.contract = contract

	callOpts := bind.CallOpts{
		Context: ctx,
	}

	blockWaitPeriod, err := wr.contract.BLOCKWAITPERIOD(&callOpts)
	if err != nil {
		return err
	}
	wr.blockWaitPeriod = blockWaitPeriod

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

				err := wr.submit(ctx, task)
				if err != nil {
					return fmt.Errorf("submit request: %w", err)
				}
			}
		}
	})

	return nil
}

func (wr *EthereumWriter) waitForTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	for {
		receipt, err := wr.pollTransaction(ctx, tx, confirmations)
		if err != nil {
			return nil, err
		}

		if receipt != nil {
			return receipt, nil
		}

		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-time.After(500 * time.Millisecond):
		}
	}
}

func (wr *EthereumWriter) pollTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (*types.Receipt, error) {
	receipt, err := wr.conn.Client().TransactionReceipt(ctx, tx.Hash())
	if err != nil {
		if errors.Is(err, goEthereum.NotFound) {
			return nil, nil
		}
	}

	latestHeader, err := wr.conn.Client().HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, err
	}

	if latestHeader.Number.Uint64()-receipt.BlockNumber.Uint64() >= confirmations {
		return receipt, nil
	}

	return nil, nil
}

func (wr *EthereumWriter) getNewRequestEvent(receipt *types.Receipt) *beefyclient.BeefyClientNewRequest {
	for _, eventLog := range receipt.Logs {
		event, err := wr.contract.ParseNewRequest(*eventLog)
		if err != nil {
			continue
		}
		return event
	}
	return nil
}

func (wr *EthereumWriter) submit(ctx context.Context, task Request) error {
	tx, err := wr.doSubmitInitial(ctx, &task)
	if err != nil {
		log.WithError(err).Error("Failed to send initial signature commitment")
		return err
	}

	receipt, err := wr.waitForTransaction(ctx, tx, wr.blockWaitPeriod)
	if err != nil {
		return err
	}
	if receipt.Status != 1 {
		return fmt.Errorf("initial commitment transaction failed")
	}

	event := wr.getNewRequestEvent(receipt)
	if event == nil {
		return fmt.Errorf("Could not find event CommitmentVerified event")
	}

	validationID := int64(event.Id.Uint64())

	log.WithFields(log.Fields{
		"event": log.Fields{
			"requestID": event.Id.Uint64(),
			"sender":    event.Sender.Hex(),
		},
	}).Info("Initial submission successful")

	tx, err = wr.doSubmitFinal(ctx, validationID, &task)
	if err != nil {
		log.WithError(err).Error("Failed to send final signature commitment")
		return err
	}

	success, err := wr.watchTransaction(ctx, tx, 0)
	if err != nil {
		return fmt.Errorf("monitoring failed for transaction SubmitFinal (%v): %w", tx.Hash().Hex(), err)
	}
	if !success {
		return fmt.Errorf("transaction SubmitFinal failed (%v)", tx.Hash().Hex())
	}

	log.WithField("tx", tx.Hash().Hex()).Debug("Transaction SubmitFinal succeeded")

	return nil

}

func (wr *EthereumWriter) watchTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (bool, error) {
	receipt, err := wr.waitForTransaction(ctx, tx, confirmations)
	if err != nil {
		return false, err
	}
	return receipt.Status == 1, nil
}

func (wr *EthereumWriter) dynamicTipCap(ctx context.Context) (*big.Int, error) {
	const maxTipInput = 25_000_000_000 // 25 Gwei
	const tipMultiplierInput = 2.0

	maxTip := new(big.Int).SetInt64(maxTipInput)
	tipMultiplier := new(big.Float).SetFloat64(tipMultiplierInput)

	suggestedTip, err := wr.conn.Client().SuggestGasTipCap(ctx)
	if err != nil {
		return nil, fmt.Errorf("suggest tip: %w", err)
	}

	tip, _ := new(big.Float).Mul(new(big.Float).SetInt(suggestedTip), tipMultiplier).Int(nil)
	if tip.Cmp(maxTip) < 0 {
		return maxTip, nil
	}
	if tip.Cmp(suggestedTip) > 0 {
		return suggestedTip, nil
	}

	return tip, nil
}

func (wr *EthereumWriter) makeTxOpts(ctx context.Context) *bind.TransactOpts {
	chainID := wr.conn.ChainID()
	keypair := wr.conn.Keypair()

	options := bind.TransactOpts{
		From: keypair.CommonAddress(),
		Signer: func(_ common.Address, tx *types.Transaction) (*types.Transaction, error) {
			return types.SignTx(tx, types.NewLondonSigner(chainID), keypair.PrivateKey())
		},
		Context: ctx,
	}

	if wr.config.Ethereum.GasFeeCap > 0 {
		fee := big.NewInt(0)
		fee.SetUint64(wr.config.Ethereum.GasFeeCap)
		options.GasFeeCap = fee
	}

	if wr.config.Ethereum.GasTipCap > 0 {
		tip := big.NewInt(0)
		tip.SetUint64(wr.config.Ethereum.GasTipCap)
		options.GasTipCap = tip
	}

	if wr.config.Ethereum.GasLimit > 0 {
		options.GasLimit = wr.config.Ethereum.GasLimit
	}

	return &options
}

func (wr *EthereumWriter) doSubmitInitial(ctx context.Context, task *Request) (*types.Transaction, error) {
	signedValidators := []*big.Int{}
	for i, signature := range task.SignedCommitment.Signatures {
		if signature.IsSome() {
			signedValidators = append(signedValidators, big.NewInt(int64(i)))
		}
	}
	numberOfValidators := big.NewInt(int64(len(task.SignedCommitment.Signatures)))
	initialBitfield, err := wr.contract.CreateInitialBitfield(
		&bind.CallOpts{Pending: true}, signedValidators, numberOfValidators,
	)
	if err != nil {
		return nil, fmt.Errorf("create initial bitfield: %w", err)
	}

	valIndex := signedValidators[0].Int64()

	msg, err := task.MakeSubmitInitialParams(valIndex, initialBitfield)
	if err != nil {
		return nil, err
	}

	gasTipCap, err := wr.dynamicTipCap(ctx)
	if err != nil {
		return nil, err
	}

	opts := wr.makeTxOpts(ctx)
	opts.GasTipCap = gasTipCap
	tx, err := wr.contract.SubmitInitial(
		opts,
		msg.CommitmentHash,
		msg.ValidatorSetID,
		msg.ValidatorClaimsBitfield,
		msg.Proof,
	)
	if err != nil {
		return nil, fmt.Errorf("initial submit: %w", err)
	}

	var pkProofHex []string
	for _, proofItem := range msg.Proof.MerkleProof {
		pkProofHex = append(pkProofHex, "0x"+hex.EncodeToString(proofItem[:]))
	}

	log.WithFields(logrus.Fields{
		"txHash":         tx.Hash().Hex(),
		"CommitmentHash": "0x" + hex.EncodeToString(msg.CommitmentHash[:]),
		"BlockNumber":    task.SignedCommitment.Commitment.BlockNumber,
		"ValidatorSetID": task.SignedCommitment.Commitment.ValidatorSetID,
	}).Info("Transaction submitted for initial verification")

	return tx, nil
}

// doFinalSubmit sends a SubmitFinal tx to the BeefyClient contract
func (wr *EthereumWriter) doSubmitFinal(ctx context.Context, validationID int64, task *Request) (*types.Transaction, error) {
	finalBitfield, err := wr.contract.CreateFinalBitfield(
		&bind.CallOpts{Pending: true},
		big.NewInt(validationID),
	)
	if err != nil {
		return nil, fmt.Errorf("create validator bitfield: %w", err)
	}

	validatorIndices := bitfield.New(finalBitfield).Members()

	params, err := task.MakeSubmitFinalParams(validationID, validatorIndices)
	if err != nil {
		return nil, err
	}

	if task.IsHandover {
		logFields, err := wr.makeSubmitFinalHandoverLogFields(task, params)
		if err != nil {
			return nil, fmt.Errorf("logging params: %w", err)
		}

		gasTipCap, err := wr.dynamicTipCap(ctx)
		if err != nil {
			return nil, err
		}

		opts := wr.makeTxOpts(ctx)
		opts.GasTipCap = gasTipCap
		tx, err := wr.contract.SubmitFinal(
			opts,
			params.ID,
			params.Commitment,
			params.Proof,
			params.Leaf,
			params.LeafProof,
		)
		if err != nil {
			return nil, fmt.Errorf("final submission: %w", err)
		}

		log.WithField("txHash", tx.Hash().Hex()).
			WithFields(logFields).
			Info("Sent SubmitFinal transaction")

		return tx, nil
	} else { // revive:disable-line
		logFields, err := wr.makeSubmitFinalLogFields(task, params)
		if err != nil {
			return nil, fmt.Errorf("logging params: %w", err)
		}

		gasTipCap, err := wr.dynamicTipCap(ctx)
		if err != nil {
			return nil, err
		}

		opts := wr.makeTxOpts(ctx)
		opts.GasTipCap = gasTipCap
		tx, err := wr.contract.SubmitFinal0(
			opts,
			params.ID,
			params.Commitment,
			params.Proof,
		)
		if err != nil {
			return nil, fmt.Errorf("final submission: %w", err)
		}

		log.WithField("txHash", tx.Hash().Hex()).
			WithFields(logFields).
			Info("Sent SubmitFinal transaction")

		return tx, nil
	}
}
