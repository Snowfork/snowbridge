package parachain

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"reflect"
	"strings"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
	"github.com/snowfork/snowbridge/relayer/contracts/opaqueproof"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"

	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"

	log "github.com/sirupsen/logrus"
)

type EthereumWriter struct {
	config                     *SinkConfig
	conn                       *ethereum.Connection
	basicInboundChannel        *basic.BasicInboundChannel
	incentivizedInboundChannel *incentivized.IncentivizedInboundChannel
	tasks                      <-chan *Task
	abiPacker                  abi.Arguments
	abiBasicUnpacker           abi.Arguments
	abiIncentivizedUnpacker    abi.Arguments
}

func NewEthereumWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
	tasks <-chan *Task,
) (*EthereumWriter, error) {
	return &EthereumWriter{
		config:                     config,
		conn:                       conn,
		basicInboundChannel:        nil,
		incentivizedInboundChannel: nil,
		tasks:                      tasks,
	}, nil
}

func (wr *EthereumWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	var address common.Address

	address = common.HexToAddress(wr.config.Contracts.BasicInboundChannel)
	basicChannel, err := basic.NewBasicInboundChannel(address, wr.conn.Client())
	if err != nil {
		return err
	}
	wr.basicInboundChannel = basicChannel

	address = common.HexToAddress(wr.config.Contracts.IncentivizedInboundChannel)
	incentivizedChannel, err := incentivized.NewIncentivizedInboundChannel(address, wr.conn.Client())
	if err != nil {
		return err
	}
	wr.incentivizedInboundChannel = incentivizedChannel

	opaqueProofABI, err := abi.JSON(strings.NewReader(opaqueproof.OpaqueProofABI))
	if err != nil {
		return err
	}
	wr.abiPacker = opaqueProofABI.Methods["dummy"].Inputs

	basicInboundChannelABI, err := abi.JSON(strings.NewReader(basic.BasicInboundChannelABI))
	if err != nil {
		return err
	}
	wr.abiBasicUnpacker = abi.Arguments{basicInboundChannelABI.Methods["submit"].Inputs[0]}

	incentivizedInboundChannelABI, err := abi.JSON(strings.NewReader(incentivized.IncentivizedInboundChannelABI))
	if err != nil {
		return err
	}
	wr.abiIncentivizedUnpacker = abi.Arguments{incentivizedInboundChannelABI.Methods["submit"].Inputs[0]}

	eg.Go(func() error {
		err := wr.writeMessagesLoop(ctx)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return fmt.Errorf("write message loop: %w", err)
		}
		return nil
	})

	return nil
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

func (wr *EthereumWriter) writeMessagesLoop(ctx context.Context) error {
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
				return fmt.Errorf("write message: %w", err)
			}
		}
	}
}

func (wr *EthereumWriter) WriteChannel(
	options *bind.TransactOpts,
	task *Task,
) error {
	for channelID, commitment := range task.Commitments {
		if channelID.IsBasic {
			err := wr.WriteBasicChannel(
				options,
				commitment.Hash,
				commitment.Data,
				task.ParaID,
				task.ProofOutput,
			)
			if err != nil {
				return fmt.Errorf("write basic channel: %w", err)
			}
		}
		if channelID.IsIncentivized {
			err := wr.WriteIncentivizedChannel(
				options,
				commitment.Hash,
				commitment.Data,
				task.ParaID,
				task.ProofOutput,
			)
			if err != nil {
				return fmt.Errorf("write incentivized channel: %w", err)
			}
		}
	}
	return nil
}

func (wr *EthereumWriter) unpackBasic(data []byte) (basic.BasicInboundChannelMessageBundle, error) {
	marshalledData, err := wr.abiBasicUnpacker.Unpack(data)
	if err != nil {
		return basic.BasicInboundChannelMessageBundle{}, fmt.Errorf("unpack commitment data: %w", err)
	}

	bundleValue := reflect.ValueOf(marshalledData[0])
	messagesField := bundleValue.FieldByName("Messages")
	bundleMessages := []basic.BasicInboundChannelMessage{}
	for i := 0; i < messagesField.Len(); i++ {
		item := messagesField.Index(i)

		target, ok := item.FieldByName("Target").Interface().(common.Address)
		if !ok {
			return basic.BasicInboundChannelMessageBundle{}, fmt.Errorf("unpack target address")
		}

		bundleMessages = append(bundleMessages, basic.BasicInboundChannelMessage{
			Id: item.FieldByName("Id").Uint(),
			Target: target,
			Payload: item.FieldByName("Payload").Bytes(),
		})
	}

	return basic.BasicInboundChannelMessageBundle{
		Nonce: bundleValue.FieldByName("Nonce").Uint(),
		Messages: bundleMessages,
	}, nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *EthereumWriter) WriteBasicChannel(
	options *bind.TransactOpts,
	commitmentHash gsrpcTypes.H256,
	commitment []byte,
	paraID uint32,
	proof *ProofOutput,
) error {
	bundle, err := wr.unpackBasic(commitment)
	if err != nil {
		return fmt.Errorf("unpack commitment: %w", err)
	}

	paraHeadProof := opaqueproof.ParachainClientHeadProof{
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

	var merkleProofItems [][32]byte
	for _, proofItem := range proof.MMRProof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, proofItem)
	}

	finalProof := opaqueproof.ParachainClientProof{
		HeadPrefix: prefix,
		HeadSuffix: suffix,
		HeadProof:  paraHeadProof,
		LeafPartial: opaqueproof.ParachainClientMMRLeafPartial{
			Version:              uint8(proof.MMRProof.Leaf.Version),
			ParentNumber:         uint32(proof.MMRProof.Leaf.ParentNumberAndHash.ParentNumber),
			ParentHash:           proof.MMRProof.Leaf.ParentNumberAndHash.Hash,
			NextAuthoritySetID:   uint64(proof.MMRProof.Leaf.BeefyNextAuthoritySet.ID),
			NextAuthoritySetLen:  uint32(proof.MMRProof.Leaf.BeefyNextAuthoritySet.Len),
			NextAuthoritySetRoot: proof.MMRProof.Leaf.BeefyNextAuthoritySet.Root,
		},
		LeafProof: opaqueproof.MMRProof{
			Items: merkleProofItems,
			Order: proof.MMRProof.MerkleProofOrder,
		},
	}

	opaqueProof, err := wr.abiPacker.Pack(finalProof)
	if err != nil {
		return fmt.Errorf("pack proof: %w", err)
	}

	tx, err := wr.basicInboundChannel.Submit(
		options, bundle, opaqueProof,
	)
	if err != nil {
		return fmt.Errorf("send transaction BasicInboundChannel.submit: %w", err)
	}

	hasher := &keccak.Keccak256{}

	mmrLeafEncoded, err := gsrpcTypes.EncodeToBytes(proof.MMRProof.Leaf)
	if err != nil {
		return fmt.Errorf("encode MMRLeaf: %w", err)
	}
	log.WithField("txHash", tx.Hash().Hex()).
		WithField("params", wr.logFieldsForBasicSubmission(bundle, opaqueProof)).
		WithFields(log.Fields{
			"commitmentHash":       commitmentHashString,
			"MMRRoot":              proof.MMRRootHash.Hex(),
			"MMRLeafHash":          Hex(hasher.Hash(mmrLeafEncoded)),
			"merkleProofData":      proof.MerkleProofData,
			"parachainBlockNumber": proof.Header.Number,
			"beefyBlock":           proof.MMRProof.Blockhash.Hex(),
		}).
		Info("Sent transaction BasicInboundChannel.submit")

	return nil
}

func (wr *EthereumWriter) unpackIncentivized(data []byte) (incentivized.IncentivizedInboundChannelMessageBundle, error) {
	marshalledData, err := wr.abiIncentivizedUnpacker.Unpack(data)
	if err != nil {
		return incentivized.IncentivizedInboundChannelMessageBundle{}, fmt.Errorf("unpack commitment data: %w", err)
	}

	bundleValue := reflect.ValueOf(marshalledData[0])
	messagesField := bundleValue.FieldByName("Messages")
	bundleMessages := []incentivized.IncentivizedInboundChannelMessage{}
	for i := 0; i < messagesField.Len(); i++ {
		item := messagesField.Index(i)

		target, ok := item.FieldByName("Target").Interface().(common.Address)
		if !ok {
			return incentivized.IncentivizedInboundChannelMessageBundle{}, fmt.Errorf("unpack target address")
		}

		bundleMessages = append(bundleMessages, incentivized.IncentivizedInboundChannelMessage{
			Id: item.FieldByName("Id").Uint(),
			Target: target,
			Payload: item.FieldByName("Payload").Bytes(),
		})
	}

	fee, ok := bundleValue.FieldByName("Fee").Interface().(*big.Int)
	if !ok {
		return incentivized.IncentivizedInboundChannelMessageBundle{}, fmt.Errorf("unpack fee")
	}

	return incentivized.IncentivizedInboundChannelMessageBundle{
		Nonce: bundleValue.FieldByName("Nonce").Uint(),
		Fee: fee,
		Messages: bundleMessages,
	}, nil
}

func (wr *EthereumWriter) WriteIncentivizedChannel(
	options *bind.TransactOpts,
	commitmentHash gsrpcTypes.H256,
	commitment []byte,
	paraID uint32,
	proof *ProofOutput,
) error {
	bundle, err := wr.unpackIncentivized(commitment)
	if err != nil {
		return fmt.Errorf("unpack commitment: %w", err)
	}

	paraHeadProof := opaqueproof.ParachainClientHeadProof{
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

	var merkleProofItems [][32]byte
	for _, proofItem := range proof.MMRProof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, proofItem)
	}

	finalProof := opaqueproof.ParachainClientProof{
		HeadPrefix: prefix,
		HeadSuffix: suffix,
		HeadProof:  paraHeadProof,
		LeafPartial: opaqueproof.ParachainClientMMRLeafPartial{
			Version:              uint8(proof.MMRProof.Leaf.Version),
			ParentNumber:         uint32(proof.MMRProof.Leaf.ParentNumberAndHash.ParentNumber),
			ParentHash:           proof.MMRProof.Leaf.ParentNumberAndHash.Hash,
			NextAuthoritySetID:   uint64(proof.MMRProof.Leaf.BeefyNextAuthoritySet.ID),
			NextAuthoritySetLen:  uint32(proof.MMRProof.Leaf.BeefyNextAuthoritySet.Len),
			NextAuthoritySetRoot: proof.MMRProof.Leaf.BeefyNextAuthoritySet.Root,
		},
		LeafProof: opaqueproof.MMRProof{
			Items: merkleProofItems,
			Order: proof.MMRProof.MerkleProofOrder,
		},
	}

	opaqueProof, err := wr.abiPacker.Pack(finalProof)
	if err != nil {
		return fmt.Errorf("pack proof: %w", err)
	}

	tx, err := wr.incentivizedInboundChannel.Submit(
		options, bundle, opaqueProof,
	)
	if err != nil {
		return fmt.Errorf("send transaction IncentivizedInboundChannel.submit: %w", err)
	}

	hasher := &keccak.Keccak256{}

	mmrLeafEncoded, err := gsrpcTypes.EncodeToBytes(proof.MMRProof.Leaf)
	if err != nil {
		return fmt.Errorf("encode MMRLeaf: %w", err)
	}
	log.WithField("txHash", tx.Hash().Hex()).
		WithField("params", wr.logFieldsForIncentivizedSubmission(bundle, opaqueProof)).
		WithFields(log.Fields{
			"commitmentHash":       commitmentHashString,
			"MMRRoot":              proof.MMRRootHash.Hex(),
			"MMRLeafHash":          Hex(hasher.Hash(mmrLeafEncoded)),
			"merkleProofData":      proof.MerkleProofData,
			"parachainBlockNumber": proof.Header.Number,
			"beefyBlock":           proof.MMRProof.Blockhash.Hex(),
		}).
		Info("Sent transaction IncentivizedInboundChannel.submit")

	return nil
}
