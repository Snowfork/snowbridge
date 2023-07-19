package parachain

import (
	"context"
	"encoding/binary"
	"errors"
	"fmt"
	"math/big"
	"strings"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"

	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"

	goEthereum "github.com/ethereum/go-ethereum"

	log "github.com/sirupsen/logrus"
)

type EthereumWriter struct {
	config     *SinkConfig
	conn       *ethereum.Connection
	gateway    *contracts.Gateway
	tasks      <-chan *Task
	abiPacker  abi.Arguments
	gatewayABI abi.ABI
}

func NewEthereumWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
	tasks <-chan *Task,
) (*EthereumWriter, error) {
	return &EthereumWriter{
		config:  config,
		conn:    conn,
		gateway: nil,
		tasks:   tasks,
	}, nil
}

func (wr *EthereumWriter) Start(ctx context.Context, eg *errgroup.Group) error {
	address := common.HexToAddress(wr.config.Contracts.Gateway)
	gateway, err := contracts.NewGateway(address, wr.conn.Client())
	if err != nil {
		return err
	}
	wr.gateway = gateway

	gatewayABI, err := abi.JSON(strings.NewReader(contracts.GatewayABI))
	if err != nil {
		return err
	}
	wr.gatewayABI = gatewayABI

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
			err := wr.WriteChannels(ctx, options, task)
			if err != nil {
				return fmt.Errorf("write message: %w", err)
			}
		}
	}
}

func (wr *EthereumWriter) WriteChannels(
	ctx context.Context,
	options *bind.TransactOpts,
	task *Task,
) error {
	for _, proof := range *task.BasicChannelProofs {
		err := wr.WriteChannel(ctx, options, &proof, task.ProofOutput)
		if err != nil {
			return fmt.Errorf("write basic channel: %w", err)
		}
	}

	return nil
}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *EthereumWriter) WriteChannel(
	ctx context.Context,
	options *bind.TransactOpts,
	commitmentProof *MessageProof,
	proof *ProofOutput,
) error {
	message := commitmentProof.Message.IntoInboundMessage()

	convertedHeader, err := convertHeader(proof.Header)
	if err != nil {
		return fmt.Errorf("convert header: %w", err)
	}

	var merkleProofItems [][32]byte
	for _, proofItem := range proof.MMRProof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, proofItem)
	}

	verificationProof := contracts.VerificationProof{
		Header: *convertedHeader,
		HeadProof: contracts.VerificationHeadProof{
			Pos:   big.NewInt(proof.MerkleProofData.ProvenLeafIndex),
			Width: big.NewInt(int64(proof.MerkleProofData.NumberOfLeaves)),
			Proof: proof.MerkleProofData.Proof,
		},
		LeafPartial: contracts.VerificationMMRLeafPartial{
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

	tx, err := wr.gateway.SubmitInbound(
		options, message, commitmentProof.Proof.InnerHashes, verificationProof,
	)
	if err != nil {
		return fmt.Errorf("send transaction Gateway.submit: %w", err)
	}

	hasher := &keccak.Keccak256{}
	mmrLeafEncoded, err := gsrpcTypes.EncodeToBytes(proof.MMRProof.Leaf)
	if err != nil {
		return fmt.Errorf("encode MMRLeaf: %w", err)
	}
	log.WithField("txHash", tx.Hash().Hex()).
		WithField("params", wr.logFieldsForSubmission(message, commitmentProof.Proof.InnerHashes, verificationProof)).
		WithFields(log.Fields{
			"commitmentHash":       commitmentProof.Proof.Root.Hex(),
			"MMRRoot":              proof.MMRRootHash.Hex(),
			"MMRLeafHash":          Hex(hasher.Hash(mmrLeafEncoded)),
			"merkleProofData":      proof.MerkleProofData,
			"parachainBlockNumber": proof.Header.Number,
			"beefyBlock":           proof.MMRProof.Blockhash.Hex(),
			"header":               proof.Header,
		}).
		Info("Sent transaction Gateway.submit")

	receipt, err := wr.waitForTransaction(ctx, tx, 1)

	if receipt.Status != 1 {
		return fmt.Errorf("transaction failed: %s", receipt.TxHash.Hex())
	}

	for _, ev := range receipt.Logs {
		if ev.Topics[0] == wr.gatewayABI.Events["InboundMessageDispatched"].ID {
			var holder contracts.GatewayInboundMessageDispatched
			err = wr.gatewayABI.UnpackIntoInterface(&holder, "InboundMessageDispatched", ev.Data)
			if err != nil {
				return fmt.Errorf("unpack event log: %w", err)
			}
			log.WithFields(log.Fields{
				"origin":  holder.Origin,
				"nonce":   holder.Nonce,
				"success": holder.Success,
			}).Info("Message dispatched")
		}
	}

	return nil
}

func convertHeader(header gsrpcTypes.Header) (*contracts.VerificationParachainHeader, error) {
	var digestItems []contracts.VerificationDigestItem

	for _, di := range header.Digest {
		switch {
		case di.IsOther:
			digestItems = append(digestItems, contracts.VerificationDigestItem{
				Kind: big.NewInt(0),
				Data: di.AsOther,
			})
		case di.IsPreRuntime:
			consensusEngineID := make([]byte, 4)
			binary.LittleEndian.PutUint32(consensusEngineID, uint32(di.AsPreRuntime.ConsensusEngineID))
			digestItems = append(digestItems, contracts.VerificationDigestItem{
				Kind:              big.NewInt(6),
				ConsensusEngineID: *(*[4]byte)(consensusEngineID),
				Data:              di.AsPreRuntime.Bytes,
			})
		case di.IsConsensus:
			consensusEngineID := make([]byte, 4)
			binary.LittleEndian.PutUint32(consensusEngineID, uint32(di.AsConsensus.ConsensusEngineID))
			digestItems = append(digestItems, contracts.VerificationDigestItem{
				Kind:              big.NewInt(4),
				ConsensusEngineID: *(*[4]byte)(consensusEngineID),
				Data:              di.AsConsensus.Bytes,
			})
		case di.IsSeal:
			consensusEngineID := make([]byte, 4)
			binary.LittleEndian.PutUint32(consensusEngineID, uint32(di.AsSeal.ConsensusEngineID))
			digestItems = append(digestItems, contracts.VerificationDigestItem{
				Kind:              big.NewInt(5),
				ConsensusEngineID: *(*[4]byte)(consensusEngineID),
				Data:              di.AsSeal.Bytes,
			})
		default:
			return nil, fmt.Errorf("Unsupported digest item: %v", di)
		}
	}

	return &contracts.VerificationParachainHeader{
		ParentHash:     header.ParentHash,
		Number:         big.NewInt(int64(header.Number)),
		StateRoot:      header.StateRoot,
		ExtrinsicsRoot: header.ExtrinsicsRoot,
		DigestItems:    digestItems,
	}, nil
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
