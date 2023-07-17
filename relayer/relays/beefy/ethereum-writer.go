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
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/bitfield"

	log "github.com/sirupsen/logrus"
)

type EthereumWriter struct {
	config          *SinkConfig
	conn            *ethereum.Connection
	contract        *contracts.BeefyClient
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
	contract, err := contracts.NewBeefyClient(address, wr.conn.Client())
	if err != nil {
		return fmt.Errorf("create beefy client: %w", err)
	}
	wr.contract = contract

	callOpts := bind.CallOpts{
		Context: ctx,
	}
	blockWaitPeriod, err := wr.contract.RandaoCommitDelay(&callOpts)
	if err != nil {
		return fmt.Errorf("create randao commit delay: %w", err)
	}
	wr.blockWaitPeriod = blockWaitPeriod.Uint64()
	log.WithField("randaoCommitDelay", wr.blockWaitPeriod).Trace("Fetched randaoCommitDelay")

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

func (wr *EthereumWriter) submit(ctx context.Context, task Request) error {
	// Initial submission
	tx, initialBitfield, err := wr.doSubmitInitial(ctx, &task)
	if err != nil {
		log.WithError(err).Error("Failed to send initial signature commitment")
		return err
	}

	// Wait RandaoCommitDelay before submit CommitPrevRandao to prevent attacker from manipulating committee memberships
	// Details in https://eth2book.info/altair/part3/config/preset/#max_seed_lookahead
	receipt, err := wr.waitForTransaction(ctx, tx, wr.blockWaitPeriod+1)
	if err != nil {
		log.WithError(err).Error("Failed to wait for RandaoCommitDelay")
		return err
	}
	if receipt.Status != 1 {
		return fmt.Errorf("initial commitment transaction failed, status (%v), logs (%v)", receipt.Status, receipt.Logs)
	}

	commitmentHash, err := task.CommitmentHash()
	if err != nil {
		return fmt.Errorf("generate commitment hash: %w", err)
	}

	// Commit PrevRandao which will be used as seed to randomly select subset of validators
	// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/core/packages/contracts/contracts/BeefyClient.sol#L446-L447
	tx, err = wr.contract.CommitPrevRandao(
		wr.makeTxOpts(ctx),
		*commitmentHash,
	)
	receipt, err = wr.waitForTransaction(ctx, tx, 1)
	if err != nil {
		return err
	}
	if receipt.Status != 1 {
		return fmt.Errorf("commitmentPrevRandao transaction failed")
	}

	// Final submission
	tx, err = wr.doSubmitFinal(ctx, *commitmentHash, initialBitfield, &task)
	if err != nil {
		log.WithError(err).Error("Failed to send final signature commitment")
		return err
	}

	success, err := wr.watchTransaction(ctx, tx, 0)
	if err != nil {
		return fmt.Errorf("monitoring failed for transaction SubmitFinal (%v): %w", tx.Hash().Hex(), err)
	}
	if !success {
		return fmt.Errorf("transaction SubmitFinal failed (%v), handover (%v)", tx.Hash().Hex(), task.IsHandover)
	}

	log.WithFields(logrus.Fields{"tx": tx.Hash().Hex(), "handover": task.IsHandover, "blockNumber": task.SignedCommitment.Commitment.BlockNumber}).Debug("Transaction SubmitFinal succeeded")

	return nil

}

func (wr *EthereumWriter) watchTransaction(ctx context.Context, tx *types.Transaction, confirmations uint64) (bool, error) {
	receipt, err := wr.waitForTransaction(ctx, tx, confirmations)
	if err != nil {
		return false, err
	}
	return receipt.Status == 1, nil
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

func (wr *EthereumWriter) doSubmitInitial(ctx context.Context, task *Request) (*types.Transaction, []*big.Int, error) {
	signedValidators := []*big.Int{}
	for i, signature := range task.SignedCommitment.Signatures {
		if signature.IsSome() {
			signedValidators = append(signedValidators, big.NewInt(int64(i)))
		}
	}
	numberOfValidators := big.NewInt(int64(len(task.SignedCommitment.Signatures)))
	initialBitfield, err := wr.contract.CreateInitialBitfield(
		&bind.CallOpts{
			Pending: true,
			From:    wr.conn.Keypair().CommonAddress(),
		},
		signedValidators, numberOfValidators,
	)
	if err != nil {
		return nil, nil, fmt.Errorf("create initial bitfield: %w", err)
	}

	// Pick first validator who signs beefy commitment
	valIndex := signedValidators[0].Int64()

	msg, err := task.MakeSubmitInitialParams(valIndex, initialBitfield)
	if err != nil {
		return nil, nil, err
	}

	var pkProofHex []string
	for _, proofItem := range msg.Proof.Proof {
		pkProofHex = append(pkProofHex, "0x"+hex.EncodeToString(proofItem[:]))
	}

	var tx *types.Transaction
	if task.IsHandover {
		tx, err = wr.contract.SubmitInitialWithHandover(
			wr.makeTxOpts(ctx),
			msg.Commitment,
			msg.Bitfield,
			msg.Proof,
		)
		if err != nil {
			return nil, nil, fmt.Errorf("initial submit with handover: %w", err)
		}
	} else {
		tx, err = wr.contract.SubmitInitial(
			wr.makeTxOpts(ctx),
			msg.Commitment,
			msg.Bitfield,
			msg.Proof,
		)
		if err != nil {
			return nil, nil, fmt.Errorf("initial submit: %w", err)
		}
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
		"HandOver":       task.IsHandover,
	}).Info("Transaction submitted for initial verification")

	return tx, initialBitfield, nil
}

// doFinalSubmit sends a SubmitFinal tx to the BeefyClient contract
func (wr *EthereumWriter) doSubmitFinal(ctx context.Context, commitmentHash [32]byte, initialBitfield []*big.Int, task *Request) (*types.Transaction, error) {
	finalBitfield, err := wr.contract.CreateFinalBitfield(
		&bind.CallOpts{
			Pending: true,
			From:    wr.conn.Keypair().CommonAddress(),
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

	if task.IsHandover {
		logFields, err := wr.makeSubmitFinalHandoverLogFields(task, params)
		if err != nil {
			return nil, fmt.Errorf("logging params: %w", err)
		}

		// In Handover mode except for the validator proof to verify commitment signature
		// will also add mmr leaf proof which contains nextAuthoritySet to verify against mmr root
		// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/core/packages/contracts/contracts/BeefyClient.sol#L342-L350
		tx, err := wr.contract.SubmitFinalWithHandover(
			wr.makeTxOpts(ctx),
			params.Commitment,
			params.Bitfield,
			params.Proofs,
			params.Leaf,
			params.LeafProof,
			params.LeafProofOrder,
		)
		if err != nil {
			return nil, fmt.Errorf("final submission: %w", err)
		}

		log.WithField("txHash", tx.Hash().Hex()).
			WithFields(logFields).
			Info("Sent SubmitFinalWithHandover transaction")

		return tx, nil
	} else { // revive:disable-line
		logFields, err := wr.makeSubmitFinalLogFields(task, params)
		if err != nil {
			return nil, fmt.Errorf("logging params: %w", err)
		}

		tx, err := wr.contract.SubmitFinal(
			wr.makeTxOpts(ctx),
			params.Commitment,
			params.Bitfield,
			params.Proofs,
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
