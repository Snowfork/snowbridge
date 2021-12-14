package parachain

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"strings"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"

	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"

	log "github.com/sirupsen/logrus"
)

type EthereumChannelWriter struct {
	config                     *SinkConfig
	conn                       *ethereum.Connection
	basicInboundChannel        *basic.BasicInboundChannel
	incentivizedInboundChannel *incentivized.IncentivizedInboundChannel
	tasks                      <-chan *Task
}

func NewEthereumChannelWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
	tasks <-chan *Task,
) (*EthereumChannelWriter, error) {
	return &EthereumChannelWriter{
		config:                     config,
		conn:                       conn,
		basicInboundChannel:        nil,
		incentivizedInboundChannel: nil,
		tasks:                      tasks,
	}, nil
}

func (wr *EthereumChannelWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	var address common.Address

	address = common.HexToAddress(wr.config.Contracts.BasicInboundChannel)
	basic, err := basic.NewBasicInboundChannel(address, wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.basicInboundChannel = basic

	address = common.HexToAddress(wr.config.Contracts.IncentivizedInboundChannel)
	incentivized, err := incentivized.NewIncentivizedInboundChannel(address, wr.conn.GetClient())
	if err != nil {
		return err
	}
	wr.incentivizedInboundChannel = incentivized

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

func (wr *EthereumChannelWriter) makeTxOpts(ctx context.Context) *bind.TransactOpts {
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

func (wr *EthereumChannelWriter) writeMessagesLoop(ctx context.Context) error {
	options := wr.makeTxOpts(ctx)
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case task, ok := <-wr.tasks:
			if !ok {
				return nil
			}
			err := wr.WriteChannel(options, task)
			if err != nil {
				log.WithError(err).Error("Error submitting message to ethereum")
				return err
			}
		}
	}
}

func (wr *EthereumChannelWriter) WriteChannel(
	options *bind.TransactOpts,
	task *Task,
) error {
	for channelID, commitment := range task.Commitments {
		if channelID.IsBasic {
			messages, ok := commitment.Data.(parachain.BasicOutboundChannelMessages)
			if !ok {
				return fmt.Errorf("Invalid commitment message data")
			}
			err := wr.WriteBasicChannel(
				options,
				commitment.Hash,
				messages,
				task.ParaID,
				task.Header,
				task.ProofOutput,
			)
			if err != nil {
				log.WithError(err).Error("Failed to write to basic channel")
				return err
			}
		}
		if channelID.IsIncentivized {
			messages, ok := commitment.Data.(parachain.IncentivizedOutboundChannelMessages)
			if !ok {
				return fmt.Errorf("Invalid commitment message data")
			}
			err := wr.WriteIncentivizedChannel(
				options,
				commitment.Hash,
				messages,
				task.ParaID,
				task.Header,
				task.ProofOutput,
			)
			if err != nil {
				log.WithError(err).Error("Failed to write to incentivized channel")
				return err
			}
		}
	}
	return nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *EthereumChannelWriter) WriteBasicChannel(
	options *bind.TransactOpts,
	commitmentHash gsrpcTypes.H256,
	commitment parachain.BasicOutboundChannelMessages,
	paraID uint32,
	paraHead *gsrpcTypes.Header,
	proof *ProofOutput,
) error {
	messages := commitment.IntoInboundMessages()

	paraHeadProof := basic.ParachainLightClientParachainHeadProof{
		Pos:   big.NewInt(int64(proof.MerkleProofData.ProvenLeafIndex)),
		Width: big.NewInt(int64(proof.MerkleProofData.NumberOfLeaves)),
		Proof: proof.MerkleProofData.Proof,
	}

	ownParachainHeadBytes := proof.MerkleProofData.ProvenPreLeaf
	ownParachainHeadBytesString := hex.EncodeToString(ownParachainHeadBytes)
	commitmentHashString := hex.EncodeToString(commitmentHash[:])
	prefixSuffix := strings.Split(ownParachainHeadBytesString, commitmentHashString)
	if len(prefixSuffix) != 2 {
		return errors.New("error splitting parachain header into prefix and suffix")
	}
	paraIDHex, err := gsrpcTypes.EncodeToHexString(paraID)
	if err != nil {
		return err
	}
	prefixWithoutParaID := strings.TrimPrefix(prefixSuffix[0], strings.TrimPrefix(paraIDHex, "0x"))
	prefix, err := hex.DecodeString(prefixWithoutParaID)
	if err != nil {
		return err
	}
	suffix, err := hex.DecodeString(prefixSuffix[1])
	if err != nil {
		return err
	}

	paraVerifyInput := basic.ParachainLightClientParachainVerifyInput{
		OwnParachainHeadPrefixBytes: prefix,
		OwnParachainHeadSuffixBytes: suffix,
		ParachainHeadProof:          paraHeadProof,
	}

	beefyMMRLeafPartial := basic.ParachainLightClientBeefyMMRLeafPartial{
		Version:              uint8(proof.MMRProof.Leaf.Version),
		ParentNumber:         uint32(proof.MMRProof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           proof.MMRProof.Leaf.ParentNumberAndHash.Hash,
		NextAuthoritySetId:   uint64(proof.MMRProof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(proof.MMRProof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: proof.MMRProof.Leaf.BeefyNextAuthoritySet.Root,
	}

	var merkleProofItems [][32]byte
	for _, proofItem := range proof.MMRProof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, proofItem)
	}

	simplifiedMMRProof := basic.SimplifiedMMRProof{
		MerkleProofItems:         merkleProofItems,
		MerkleProofOrderBitField: proof.MMRProof.MerkleProofOrder,
	}

	err = wr.logBasicTx(
		messages, paraVerifyInput,
		beefyMMRLeafPartial, simplifiedMMRProof,
		paraHead, proof.MerkleProofData, proof.MMRProof.Leaf,
		commitmentHash, paraID, proof.MMRRootHash,
	)
	if err != nil {
		log.WithError(err).Error("Failed to log transaction input")
		return err
	}

	tx, err := wr.basicInboundChannel.Submit(
		options, messages, paraVerifyInput,
		beefyMMRLeafPartial,
		simplifiedMMRProof,
	)
	if err != nil {
		log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	log.WithFields(log.Fields{
		"txHash":  tx.Hash().Hex(),
		"channel": "Basic",
	}).Info("Transaction submitted")

	return nil
}

func (wr *EthereumChannelWriter) WriteIncentivizedChannel(
	options *bind.TransactOpts,
	commitmentHash gsrpcTypes.H256,
	commitment parachain.IncentivizedOutboundChannelMessages,
	paraID uint32,
	paraHead *gsrpcTypes.Header,
	proof *ProofOutput,
) error {
	messages := commitment.IntoInboundMessages()

	paraHeadProof := incentivized.ParachainLightClientParachainHeadProof{
		Pos:   big.NewInt(int64(proof.MerkleProofData.ProvenLeafIndex)),
		Width: big.NewInt(int64(proof.MerkleProofData.NumberOfLeaves)),
		Proof: proof.MerkleProofData.Proof,
	}

	ownParachainHeadBytes := proof.MerkleProofData.ProvenPreLeaf
	ownParachainHeadBytesString := hex.EncodeToString(ownParachainHeadBytes)
	commitmentHashString := hex.EncodeToString(commitmentHash[:])
	prefixSuffix := strings.Split(ownParachainHeadBytesString, commitmentHashString)
	if len(prefixSuffix) != 2 {
		return errors.New("error splitting parachain header into prefix and suffix")
	}
	paraIDHex, err := gsrpcTypes.EncodeToHexString(paraID)
	if err != nil {
		return err
	}
	prefixWithoutParaID := strings.TrimPrefix(prefixSuffix[0], strings.TrimPrefix(paraIDHex, "0x"))
	prefix, err := hex.DecodeString(prefixWithoutParaID)
	if err != nil {
		return err
	}
	suffix, err := hex.DecodeString(prefixSuffix[1])
	if err != nil {
		return err
	}

	paraVerifyInput := incentivized.ParachainLightClientParachainVerifyInput{
		OwnParachainHeadPrefixBytes: prefix,
		OwnParachainHeadSuffixBytes: suffix,
		ParachainHeadProof:          paraHeadProof,
	}

	beefyMMRLeafPartial := incentivized.ParachainLightClientBeefyMMRLeafPartial{
		Version:              uint8(proof.MMRProof.Leaf.Version),
		ParentNumber:         uint32(proof.MMRProof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           proof.MMRProof.Leaf.ParentNumberAndHash.Hash,
		NextAuthoritySetId:   uint64(proof.MMRProof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(proof.MMRProof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: proof.MMRProof.Leaf.BeefyNextAuthoritySet.Root,
	}

	var merkleProofItems [][32]byte
	for _, proofItem := range proof.MMRProof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, proofItem)
	}

	simplifiedMMRProof := incentivized.SimplifiedMMRProof{
		MerkleProofItems:         merkleProofItems,
		MerkleProofOrderBitField: proof.MMRProof.MerkleProofOrder,
	}

	err = wr.logIncentivizedTx(
		messages, paraVerifyInput,
		beefyMMRLeafPartial, simplifiedMMRProof,
		paraHead, proof.MerkleProofData, proof.MMRProof.Leaf,
		commitmentHash, paraID, proof.MMRRootHash,
	)
	if err != nil {
		log.WithError(err).Error("Failed to log transaction input")
		return err
	}

	tx, err := wr.incentivizedInboundChannel.Submit(
		options, messages,
		paraVerifyInput, beefyMMRLeafPartial,
		simplifiedMMRProof)
	if err != nil {
		log.WithError(err).Error("Failed to submit transaction")
		return err
	}

	log.WithFields(log.Fields{
		"txHash":  tx.Hash().Hex(),
		"channel": "Incentivized",
	}).Info("Transaction submitted")

	return nil
}
