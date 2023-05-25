package parachain

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"strings"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"

	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"

	log "github.com/sirupsen/logrus"
)

type EthereumWriter struct {
	config           *SinkConfig
	conn             *ethereum.Connection
	inboundQueue     *contracts.InboundQueue
	tasks            <-chan *Task
	abiPacker        abi.Arguments
	abiBasicUnpacker abi.Arguments
}

func NewEthereumWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
	tasks <-chan *Task,
) (*EthereumWriter, error) {
	return &EthereumWriter{
		config:       config,
		conn:         conn,
		inboundQueue: nil,
		tasks:        tasks,
	}, nil
}

func (wr *EthereumWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	address := common.HexToAddress(wr.config.Contracts.InboundQueue)
	basicChannel, err := contracts.NewInboundQueue(address, wr.conn.Client())
	if err != nil {
		return err
	}
	wr.inboundQueue = basicChannel

	opaqueProofABI, err := abi.JSON(strings.NewReader(contracts.OpaqueProofABI))
	if err != nil {
		return err
	}
	wr.abiPacker = opaqueProofABI.Methods["dummy"].Inputs

	inboundQueueABI, err := abi.JSON(strings.NewReader(contracts.InboundQueueABI))
	if err != nil {
		return err
	}
	wr.abiBasicUnpacker = abi.Arguments{inboundQueueABI.Methods["submit"].Inputs[0]}

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
			err := wr.WriteChannels(options, task)
			if err != nil {
				return fmt.Errorf("write message: %w", err)
			}
		}
	}
}

func (wr *EthereumWriter) WriteChannels(
	options *bind.TransactOpts,
	task *Task,
) error {
	for _, proof := range *task.BasicChannelProofs {
		err := wr.WriteBasicChannel(options, &proof, task.ProofInput.ParaID, task.ProofOutput)
		if err != nil {
			return fmt.Errorf("write basic channel: %w", err)
		}
	}

	return nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *EthereumWriter) WriteBasicChannel(
	options *bind.TransactOpts,
	commitmentProof *MessageProof,
	paraID uint32,
	proof *ProofOutput,
) error {
	message := commitmentProof.Message.IntoInboundMessage()

	paraHeadProof := contracts.ParachainClientHeadProof{
		Pos:   big.NewInt(int64(proof.MerkleProofData.ProvenLeafIndex)),
		Width: big.NewInt(int64(proof.MerkleProofData.NumberOfLeaves)),
		Proof: proof.MerkleProofData.Proof,
	}

	// Split message commit hash from parachain header since added as digest log
	// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/parachain/pallets/incentivized-channel/src/outbound/mod.rs#L238-L242
	ownParachainHeadBytes := proof.MerkleProofData.ProvenPreLeaf
	ownParachainHeadBytesString := hex.EncodeToString(ownParachainHeadBytes)
	commitmentHashString := hex.EncodeToString(commitmentProof.Proof.Root[:])
	// Trick here is that in parachain header only commitmentHash is required to verify
	// so just split to some unknown prefix and suffix in order to reconstruct later
	// https://github.com/Snowfork/snowbridge/blob/75a475cbf8fc8e13577ad6b773ac452b2bf82fbb/core/packages/contracts/contracts/ParachainClient.sol#L50-L54
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

	finalProof := contracts.ParachainClientProof{
		HeadPrefix: prefix,
		HeadSuffix: suffix,
		HeadProof:  paraHeadProof,
		LeafPartial: contracts.ParachainClientMMRLeafPartial{
			Version:              uint8(proof.MMRProof.Leaf.Version),
			ParentNumber:         uint32(proof.MMRProof.Leaf.ParentNumberAndHash.ParentNumber),
			ParentHash:           proof.MMRProof.Leaf.ParentNumberAndHash.Hash,
			NextAuthoritySetID:   uint64(proof.MMRProof.Leaf.BeefyNextAuthoritySet.ID),
			NextAuthoritySetLen:  uint32(proof.MMRProof.Leaf.BeefyNextAuthoritySet.Len),
			NextAuthoritySetRoot: proof.MMRProof.Leaf.BeefyNextAuthoritySet.Root,
		},
		LeafProof:      merkleProofItems,
		LeafProofOrder: new(big.Int).SetUint64(proof.MMRProof.MerkleProofOrder),
	}

	opaqueProof, err := wr.abiPacker.Pack(finalProof)
	if err != nil {
		return fmt.Errorf("pack proof: %w", err)
	}

	tx, err := wr.inboundQueue.Submit(
		options, message, commitmentProof.Proof.InnerHashes, opaqueProof,
	)
	if err != nil {
		return fmt.Errorf("send transaction InboundQueue.submit: %w", err)
	}

	hasher := &keccak.Keccak256{}

	mmrLeafEncoded, err := gsrpcTypes.EncodeToBytes(proof.MMRProof.Leaf)
	if err != nil {
		return fmt.Errorf("encode MMRLeaf: %w", err)
	}
	log.WithField("txHash", tx.Hash().Hex()).
		WithField("params", wr.logFieldsForBasicSubmission(message, commitmentProof.Proof.InnerHashes, opaqueProof)).
		WithFields(log.Fields{
			"commitmentHash":       commitmentHashString,
			"MMRRoot":              proof.MMRRootHash.Hex(),
			"MMRLeafHash":          Hex(hasher.Hash(mmrLeafEncoded)),
			"merkleProofData":      proof.MerkleProofData,
			"parachainBlockNumber": proof.Header.Number,
			"beefyBlock":           proof.MMRProof.Blockhash.Hex(),
		}).
		Info("Sent transaction InboundQueue.submit")

	return nil
}
