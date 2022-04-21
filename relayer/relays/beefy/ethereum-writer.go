package beefy

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"strconv"
	"time"

	"golang.org/x/sync/errgroup"

	goEthereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"

	log "github.com/sirupsen/logrus"
)

type EthereumWriter struct {
	config           *SinkConfig
	conn             *ethereum.Connection
	beefyLightClient *beefylightclient.Contract
	tasks            <-chan Task
	someTasks        chan Task

	blockWaitPeriod uint64
	validatorSetID  uint64
}

func NewEthereumWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
	tasks <-chan Task,
) *EthereumWriter {
	return &EthereumWriter{
		config: config,
		conn:   conn,
		tasks:  tasks,
		someTasks: make(chan Task),
	}
}

func (wr *EthereumWriter) Start(ctx context.Context, eg *errgroup.Group) (uint64, error) {

	address := common.HexToAddress(wr.config.Contracts.BeefyLightClient)
	beefyLightClientContract, err := beefylightclient.NewContract(address, wr.conn.GetClient())
	if err != nil {
		return 0, err
	}
	wr.beefyLightClient = beefyLightClientContract

	latestBeefyBlock, err := wr.beefyLightClient.ContractCaller.LatestBeefyBlock(&bind.CallOpts{
		Pending: false,
		Context: ctx,
	})
	if err != nil {
		return latestBeefyBlock, err
	}

	// Fetch BLOCK_WAIT_PERIOD from light client bridge contract
	blockWaitPeriod, err := wr.beefyLightClient.ContractCaller.BLOCKWAITPERIOD(nil)
	if err != nil {
		return 0, err
	}
	wr.blockWaitPeriod = blockWaitPeriod

	// launch task filterer
	eg.Go(func() error {
		defer close(wr.someTasks)
		err := wr.filterTasks(ctx)
		log.WithField("reason", err).Info("Shutting down task filter")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})

	// launch task processor
	eg.Go(func() error {
		err := wr.processAllMessages(ctx)
		log.WithField("reason", err).Info("Shutting down ethereum writer")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})

	return latestBeefyBlock, nil
}

func (wr *EthereumWriter) processAllMessages(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case task, ok := <-wr.someTasks:
			if !ok {
				return nil
			}

			err := wr.processMessage(ctx, task)
			if err != nil {
				return err
			}
		}
	}
}

func (wr *EthereumWriter) filterTasks(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case task, ok := <-wr.tasks:
			if !ok {
				return nil
			}

			log.WithFields(
				log.Fields{
					"ValidatorSetID": task.SignedCommitment.Commitment.ValidatorSetID,
					"NextValidatorSetID": task.Proof.Leaf.BeefyNextAuthoritySet.ID,
				},
			).Info("Processing commitment")

			// if task.SignedCommitment.Commitment.ValidatorSetID == wr.validatorSetID {
			// 	select {
			// 	case wr.someTasks <- task:
			// 	default:
			// 		// drop task if it can't be processed right now
			// 		log.WithField("validatorSetId", task.SignedCommitment.Commitment.ValidatorSetID).Info("Discarded commitment")
			// 	}
			// } else {
			// 	select {
			// 	case <-ctx.Done():
			// 		return ctx.Err()
			// 	case wr.someTasks <- task:
			// 	}
			// }
		}
	}
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
	receipt, err := wr.conn.GetClient().TransactionReceipt(ctx, tx.Hash())
	if err != nil {
		if errors.Is(err, goEthereum.NotFound) {
			return nil, nil
		}
	}

	latestHeader, err := wr.conn.GetClient().HeaderByNumber(ctx, nil)
	if err != nil {
		return nil, err
	}

	if latestHeader.Number.Uint64()-receipt.BlockNumber.Uint64() >= confirmations {
		return receipt, nil
	}

	return nil, nil
}

func (wr *EthereumWriter) getContractCommitmentVerified(receipt *types.Receipt) *beefylightclient.ContractCommitmentVerified {
	for _, eventLog := range receipt.Logs {
		event, err := wr.beefyLightClient.ParseCommitmentVerified(*eventLog)
		if err != nil {
			continue
		}
		return event
	}
	return nil
}

func (wr *EthereumWriter) processMessage(ctx context.Context, task Task) error {
	tx, err := wr.WriteInitialSignatureCommitment(ctx, &task)
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

	event := wr.getContractCommitmentVerified(receipt)
	if event == nil {
		return fmt.Errorf("Could not find event CommitmentVerified event")
	}
	if event.Phase != 0 {
		return fmt.Errorf("Could not find event CommitmentVerified event with phase INIT")
	}

	log.WithFields(log.Fields{
		"event": log.Fields{
			"ID":             event.Id.Uint64(),
			"Phase":          event.Phase,
			"CommitmentHash": "0x" + hex.EncodeToString(event.CommitmentHash[:]),
			"Prover":         event.Prover.Hex(),
		},
	}).Info("Initial Verification Successful")

	task.ValidationID = int64(event.Id.Uint64())

	tx, err = wr.WriteFinalSignatureCommitment(ctx, &task)
	if err != nil {
		log.WithError(err).Error("Failed to send final signature commitment")
		return err
	}

	var leafUpdateTx *types.Transaction

	nextValidatorSet, err := wr.beefyLightClient.NextValidatorSet(&bind.CallOpts{Pending: true})
	if err != nil {
		return err
	}

	if uint64(task.Proof.Leaf.BeefyNextAuthoritySet.ID) == nextValidatorSet.Id.Uint64()+1 {
		msg, err := task.MakeLeafUpdate()
		if err != nil {
			return err
		}

		options := wr.makeTxOpts(ctx)

		wr.LogLeafUpdate(task, msg)
		leafUpdateTx, err = wr.beefyLightClient.UpdateValidatorSet(options, msg.Leaf, msg.Proof)
		if err != nil {
			return err
		}
	}

	receipt, err = wr.waitForTransaction(ctx, tx, wr.config.DescendantsUntilFinal)
	if err != nil {
		return err
	}

	if receipt.Status != 1 {
		log.WithField("tx", tx.Hash().Hex()).Error("transaction failed")
		return fmt.Errorf("final commitment transaction failed")
	}

	event = wr.getContractCommitmentVerified(receipt)
	if event == nil {
		return fmt.Errorf("Could not find event CommitmentVerified event")
	}
	if event.Phase != 1 {
		return fmt.Errorf("Could not find event CommitmentVerified event with phase FINAL")
	}

	log.WithFields(log.Fields{
		"event": log.Fields{
			"ID":             event.Id.Uint64(),
			"Phase":          event.Phase,
			"CommitmentHash": "0x" + hex.EncodeToString(event.CommitmentHash[:]),
			"Prover":         event.Prover.Hex(),
		},
	}).Info("Final Verification Successful")

	if leafUpdateTx != nil {
		receipt, err = wr.waitForTransaction(ctx, tx, wr.config.DescendantsUntilFinal)
		if err != nil {
			return err
		}

		if receipt.Status != 1 {
			log.WithField("tx", tx.Hash().Hex()).Error("updateLeaf transaction failed")
			return fmt.Errorf("updateLeaf Transaction failed")
		}

		log.Info("Update ValidatorSet success")
	}

	return nil

}

func (wr *EthereumWriter) makeTxOpts(ctx context.Context) *bind.TransactOpts {
	chainID := wr.conn.ChainID()
	keypair := wr.conn.GetKP()

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

func (wr *EthereumWriter) WriteInitialSignatureCommitment(ctx context.Context, task *Task) (*types.Transaction, error) {
	contract := wr.beefyLightClient
	if contract == nil {
		return nil, fmt.Errorf("unknown contract")
	}

	signedValidators := []*big.Int{}
	for i, signature := range task.SignedCommitment.Signatures {
		if signature.IsSome() {
			signedValidators = append(signedValidators, big.NewInt(int64(i)))
		}
	}
	numberOfValidators := big.NewInt(int64(len(task.SignedCommitment.Signatures)))
	initialBitfield, err := contract.CreateInitialBitfield(
		&bind.CallOpts{Pending: true}, signedValidators, numberOfValidators,
	)
	if err != nil {
		log.WithError(err).Error("Failed to create initial validator bitfield")
		return nil, err
	}

	valIndex := signedValidators[0].Int64()

	msg, err := task.MakeInitialSignatureCommitment(valIndex, initialBitfield)
	if err != nil {
		return nil, err
	}

	options := wr.makeTxOpts(ctx)

	tx, err := contract.NewSignatureCommitment(options, msg.CommitmentHash, msg.ValidatorSetID,
		msg.ValidatorClaimsBitfield, msg.ValidatorSignatureCommitment,
		msg.ValidatorPosition, msg.ValidatorPublicKey, msg.ValidatorPublicKeyMerkleProof)
	if err != nil {
		log.WithError(err).Error("Failed to submit transaction for initial signature commitment")
		return nil, err
	}

	var pkProofHex []string
	for _, proofItem := range msg.ValidatorPublicKeyMerkleProof {
		pkProofHex = append(pkProofHex, "0x"+hex.EncodeToString(proofItem[:]))
	}

	log.WithFields(logrus.Fields{
		"txHash":                        tx.Hash().Hex(),
		"CommitmentHash":                "0x" + hex.EncodeToString(msg.CommitmentHash[:]),
		"ValidatorSignatureCommitment":  "0x" + hex.EncodeToString(msg.ValidatorSignatureCommitment),
		"ValidatorPublicKey":            msg.ValidatorPublicKey.Hex(),
		"ValidatorPublicKeyMerkleProof": pkProofHex,
		"BlockNumber":                   task.SignedCommitment.Commitment.BlockNumber,
		"ValidatorSetID":                task.SignedCommitment.Commitment.ValidatorSetID,
	}).Info("Transaction submitted for initial signature commitment")

	return tx, nil
}

func BitfieldToString(bitfield []*big.Int) string {
	bitfieldString := ""
	for _, bitfieldInt := range bitfield {
		bits := strconv.FormatInt(bitfieldInt.Int64(), 2)

		// add bits from this int at leftmost position
		bitfieldString = bits + bitfieldString

		// pad to 256 bits to include missing validators
		for bitsLength := len(bits); bitsLength < 256; bitsLength++ {
			bitfieldString = "0" + bitfieldString
		}
	}
	return bitfieldString
}

// WriteCompleteSignatureCommitment sends a CompleteSignatureCommitment tx to the BeefyLightClient contract
func (wr *EthereumWriter) WriteFinalSignatureCommitment(ctx context.Context, task *Task) (*types.Transaction, error) {
	contract := wr.beefyLightClient
	if contract == nil {
		return nil, fmt.Errorf("unknown contract")
	}

	randomBitfield, err := contract.CreateRandomBitfield(
		&bind.CallOpts{Pending: true},
		big.NewInt(task.ValidationID),
	)
	if err != nil {
		log.WithError(err).Error("Failed to get random validator bitfield")
		return nil, err
	}

	bitfield := BitfieldToString(randomBitfield)

	msg, err := task.MakeFinalSignatureCommitment(bitfield)
	if err != nil {
		return nil, err
	}

	options := wr.makeTxOpts(ctx)

	validatorProof := beefylightclient.BeefyLightClientValidatorProof{
		Signatures:            msg.Signatures,
		Positions:             msg.ValidatorPositions,
		PublicKeys:            msg.ValidatorPublicKeys,
		PublicKeyMerkleProofs: msg.ValidatorPublicKeyMerkleProofs,
	}

	err = wr.LogFinal(task, msg)
	if err != nil {
		return nil, err
	}

	tx, err := contract.CompleteSignatureCommitment(options,
		msg.ID,
		msg.Commitment,
		validatorProof)

	if err != nil {
		log.WithError(err).Error("Failed to submit transaction for final signature commitment")
		return nil, err
	}

	log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("Transaction submitted for final signature commitment")

	return tx, nil
}
