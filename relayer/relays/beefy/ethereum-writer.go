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
	ethereumConn     *ethereum.Connection
	store            *Database
	beefyLightClient *beefylightclient.Contract
	databaseMessages chan<- DatabaseCmd
	tasks            <-chan Task
}

func NewEthereumWriter(
	config *SinkConfig,
	ethereumConn *ethereum.Connection,
	store *Database,
	databaseMessages chan<- DatabaseCmd,
	tasks <-chan Task,
) *EthereumWriter {
	return &EthereumWriter{
		config:           config,
		ethereumConn:     ethereumConn,
		store:            store,
		databaseMessages: databaseMessages,
		tasks:            tasks,
	}
}

func (wr *EthereumWriter) Start(ctx context.Context, eg *errgroup.Group) error {

	address := common.HexToAddress(wr.config.Contracts.BeefyLightClient)
	beefyLightClientContract, err := beefylightclient.NewContract(address, wr.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	wr.beefyLightClient = beefyLightClientContract

	eg.Go(func() error {
		err := wr.writeMessagesLoop(ctx)
		log.WithField("reason", err).Info("Shutting down ethereum writer")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})

	return nil
}

func (wr *EthereumWriter) writeMessagesLoop(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case task, ok := <-wr.tasks:
			if !ok {
				return nil
			}
			switch task.Status {
			case CommitmentWitnessed:
				err := wr.WriteInitialSignatureCommitment(ctx, &task)
				if err != nil {
					log.WithError(err).Error("Failed to send initial signature commitment")
					return err
				}
			case ReadyToComplete:
				err := wr.WriteFinalSignatureCommitment(ctx, &task)
				if err != nil {
					log.WithError(err).Error("Failed to send complete signature commitment")
					return err
				}
			}
			// Rate-limit transaction sending to reduce the chance of transactions using the same pending nonce.
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-time.After(2 * time.Second):
			}
		}
	}
}

func (wr *EthereumWriter) makeTxOpts(ctx context.Context) *bind.TransactOpts {
	chainID := wr.ethereumConn.ChainID()
	keypair := wr.ethereumConn.GetKP()

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

func (wr *EthereumWriter) WriteInitialSignatureCommitment(ctx context.Context, task *Task) error {
	contract := wr.beefyLightClient
	if contract == nil {
		return fmt.Errorf("unknown contract")
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
		return err
	}

	valIndex := signedValidators[0].Int64()

	msg, err := task.MakeInitialSignatureCommitment(valIndex, initialBitfield)
	if err != nil {
		return err
	}

	options := wr.makeTxOpts(ctx)

	tx, err := contract.NewSignatureCommitment(options, msg.CommitmentHash,
		msg.ValidatorClaimsBitfield, msg.ValidatorSignatureCommitment,
		msg.ValidatorPosition, msg.ValidatorPublicKey, msg.ValidatorPublicKeyMerkleProof)
	if err != nil {
		log.WithError(err).Error("Failed to submit transaction for initial signature commitment")
		return err
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
	}).Info("Transaction submitted for initial signature commitment")

	task.Status = InitialVerificationTxSent
	task.InitialVerificationTx = tx.Hash()

	select {
	case <-ctx.Done():
		return ctx.Err()
	case wr.databaseMessages <- NewDatabaseCmd(task, Create, nil):
	}

	return nil
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
func (wr *EthereumWriter) WriteFinalSignatureCommitment(ctx context.Context, task *Task) error {
	contract := wr.beefyLightClient
	if contract == nil {
		return fmt.Errorf("unknown contract")
	}

	randomBitfield, err := contract.CreateRandomBitfield(
		&bind.CallOpts{Pending: true},
		big.NewInt(task.ValidationID),
	)
	if err != nil {
		log.WithError(err).Error("Failed to get random validator bitfield")
		return err
	}

	bitfield := BitfieldToString(randomBitfield)

	msg, err := task.MakeFinalSignatureCommitment(bitfield)
	if err != nil {
		return err
	}

	options := wr.makeTxOpts(ctx)

	validatorProof := beefylightclient.BeefyLightClientValidatorProof{
		Signatures:            msg.Signatures,
		Positions:             msg.ValidatorPositions,
		PublicKeys:            msg.ValidatorPublicKeys,
		PublicKeyMerkleProofs: msg.ValidatorPublicKeyMerkleProofs,
	}

	err = wr.LogBeefyFixtureDataAll(task, msg)
	if err != nil {
		return err
	}

	tx, err := contract.CompleteSignatureCommitment(options,
		msg.ID,
		msg.Commitment,
		validatorProof,
		msg.LatestMMRLeaf,
		beefylightclient.SimplifiedMMRProof{
			MerkleProofItems:         msg.SimplifiedProof.MerkleProofItems,
			MerkleProofOrderBitField: msg.SimplifiedProof.MerkleProofOrderBitField,
		})

	if err != nil {
		log.WithError(err).Error("Failed to submit transaction for final signature commitment")
		return err
	}

	log.WithFields(logrus.Fields{
		"txHash": tx.Hash().Hex(),
	}).Info("Transaction submitted for final signature commitment")

	// Update item's status in database
	instructions := map[string]interface{}{
		"status":                CompleteVerificationTxSent,
		"final_verification_tx": tx.Hash(),
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case wr.databaseMessages <- NewDatabaseCmd(task, Update, instructions):
	}

	return nil
}
