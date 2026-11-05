package parachainV3

import (
	"context"
	"encoding/binary"
	"fmt"
	"math/big"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	contractsv1 "github.com/snowfork/snowbridge/relayer/contracts/v1"

	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"
)

type EthereumWriter struct {
	config   *SinkConfig
	conn     *ethereum.Connection
	readConn *ethereum.Connection
	// dryRunConn, when set, is used for aggregate3 simulation (eth_call/eth_estimateGas)
	// instead of the broadcast conn. It points at a dry-run/forked-mainnet RPC that
	// whitelists eth_call; the broadcast RPC (e.g. flashbots) does not. nil → use conn.
	dryRunConn      *ethereum.Connection
	gateway         *contracts.Gateway
	gatewayReadOnly *contracts.Gateway
	tasks           <-chan *Task
	gatewayABI      abi.ABI
	gatewayABIV1    abi.ABI
	relayConfig     *Config
}

func NewEthereumWriter(
	config *SinkConfig,
	conn *ethereum.Connection,
	readConn *ethereum.Connection,
	dryRunConn *ethereum.Connection,
	tasks <-chan *Task,
	relayConfig *Config,
) (*EthereumWriter, error) {
	return &EthereumWriter{
		config:      config,
		conn:        conn,
		readConn:    readConn,
		dryRunConn:  dryRunConn,
		gateway:     nil,
		tasks:       tasks,
		relayConfig: relayConfig,
	}, nil
}

// dryRunClientConn returns the connection to use for aggregate3 simulation: the dedicated
// dry-run connection when it is configured and connected, otherwise the broadcast conn.
func (wr *EthereumWriter) dryRunClientConn() *ethereum.Connection {
	if wr.dryRunConn != nil && wr.dryRunConn.Client() != nil {
		return wr.dryRunConn
	}
	return wr.conn
}

func (wr *EthereumWriter) initialize() error {
	if err := wr.rebindGateways(); err != nil {
		return err
	}

	gatewayABI, err := abi.JSON(strings.NewReader(contracts.GatewayABI))
	if err != nil {
		return err
	}
	wr.gatewayABI = gatewayABI

	gatewayABIV1, err := abi.JSON(strings.NewReader(contractsv1.GatewayABI))
	if err != nil {
		return err
	}
	wr.gatewayABIV1 = gatewayABIV1

	return nil
}

// rebindGateways recreates Gateway bindings after an Ethereum Reconnect.
// LiveClient already forwards to the new inner client; rebind matches the
// E→P reconnectAndRebind path and covers any binding that cached a client.
func (wr *EthereumWriter) rebindGateways() error {
	address := common.HexToAddress(wr.config.Contracts.Gateway)
	gateway, err := contracts.NewGateway(address, wr.conn.Client())
	if err != nil {
		return err
	}
	gatewayReadOnly, err := contracts.NewGateway(address, wr.readConn.Client())
	if err != nil {
		return err
	}
	wr.gateway = gateway
	wr.gatewayReadOnly = gatewayReadOnly
	return nil
}

func (wr *EthereumWriter) commandGas(command *CommandWrapper) uint64 {
	return expectedCommandDispatchGas(command)
}

// MessageMinFeeWei is the minimum message fee in wei at gasPrice, matching relay profitability checks:
// gasPrice * (sum(command gas) + BaseMessageVerificationGas).
// SuggestRelayGasPrice returns the gas price from the writer read client (same source as per-message profitability).
func (wr *EthereumWriter) SuggestRelayGasPrice(ctx context.Context) (*big.Int, error) {
	return wr.readConn.Client().SuggestGasPrice(ctx)
}

func (wr *EthereumWriter) MessageMinFeeWei(gasPrice *big.Int, proof *MessageProof) *big.Int {
	var totalDispatchGas uint64
	for _, command := range proof.Message.OriginalMessage.Commands {
		totalDispatchGas += wr.commandGas(&command)
	}
	totalDispatchGas += BaseMessageVerificationGas

	gasFee := new(big.Int).Mul(gasPrice, new(big.Int).SetUint64(totalDispatchGas))
	return gasFee
}

func (wr *EthereumWriter) BuildV2SubmitCalldata(
	commitmentProof *MessageProof,
	proof *ProofOutput,
	rewardAddress [32]byte,
) ([]byte, error) {
	message, verificationProof, err := wr.buildV2SubmitParams(commitmentProof, proof)
	if err != nil {
		return nil, err
	}

	calldata, err := wr.gatewayABI.Pack(
		"v2_submit",
		message,
		commitmentProof.Proof.InnerHashes,
		verificationProof,
		rewardAddress,
	)
	if err != nil {
		return nil, fmt.Errorf("pack v2_submit calldata: %w", err)
	}

	return calldata, nil
}

func (wr *EthereumWriter) GatewayAddress() common.Address {
	return common.HexToAddress(wr.config.Contracts.Gateway)
}

func (wr *EthereumWriter) BuildV1SubmitCalldata(
	commitmentProof *V1MessageProof,
	proof *ProofOutput,
) ([]byte, error) {
	message, verificationProof, err := wr.buildV1SubmitParams(commitmentProof, proof)
	if err != nil {
		return nil, err
	}

	calldata, err := wr.gatewayABIV1.Pack(
		"submitV1",
		message,
		commitmentProof.Proof.InnerHashes,
		verificationProof,
	)
	if err != nil {
		return nil, fmt.Errorf("pack submitV1 calldata: %w", err)
	}

	return calldata, nil
}

func (wr *EthereumWriter) buildV2SubmitParams(
	commitmentProof *MessageProof,
	proof *ProofOutput,
) (contracts.InboundMessage, contracts.VerificationProof, error) {
	message := commitmentProof.Message.OriginalMessage.IntoInboundMessage()

	convertedHeader, err := convertHeader(proof.Header)
	if err != nil {
		return contracts.InboundMessage{}, contracts.VerificationProof{}, fmt.Errorf("convert header: %w", err)
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

	return message, verificationProof, nil
}

func (wr *EthereumWriter) buildV1SubmitParams(
	commitmentProof *V1MessageProof,
	proof *ProofOutput,
) (contractsv1.InboundMessage, contractsv1.VerificationProof, error) {
	message := commitmentProof.Message.IntoInboundMessage()

	convertedHeader, err := convertHeaderV1(proof.Header)
	if err != nil {
		return contractsv1.InboundMessage{}, contractsv1.VerificationProof{}, fmt.Errorf("convert header: %w", err)
	}

	var merkleProofItems [][32]byte
	for _, proofItem := range proof.MMRProof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, proofItem)
	}

	verificationProof := contractsv1.VerificationProof{
		Header: *convertedHeader,
		HeadProof: contractsv1.VerificationHeadProof{
			Pos:   big.NewInt(proof.MerkleProofData.ProvenLeafIndex),
			Width: big.NewInt(int64(proof.MerkleProofData.NumberOfLeaves)),
			Proof: proof.MerkleProofData.Proof,
		},
		LeafPartial: contractsv1.VerificationMMRLeafPartial{
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

	return message, verificationProof, nil
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

func convertHeaderV1(header gsrpcTypes.Header) (*contractsv1.VerificationParachainHeader, error) {
	var digestItems []contractsv1.VerificationDigestItem

	for _, di := range header.Digest {
		switch {
		case di.IsOther:
			digestItems = append(digestItems, contractsv1.VerificationDigestItem{
				Kind: big.NewInt(0),
				Data: di.AsOther,
			})
		case di.IsPreRuntime:
			consensusEngineID := make([]byte, 4)
			binary.LittleEndian.PutUint32(consensusEngineID, uint32(di.AsPreRuntime.ConsensusEngineID))
			digestItems = append(digestItems, contractsv1.VerificationDigestItem{
				Kind:              big.NewInt(6),
				ConsensusEngineID: *(*[4]byte)(consensusEngineID),
				Data:              di.AsPreRuntime.Bytes,
			})
		case di.IsConsensus:
			consensusEngineID := make([]byte, 4)
			binary.LittleEndian.PutUint32(consensusEngineID, uint32(di.AsConsensus.ConsensusEngineID))
			digestItems = append(digestItems, contractsv1.VerificationDigestItem{
				Kind:              big.NewInt(4),
				ConsensusEngineID: *(*[4]byte)(consensusEngineID),
				Data:              di.AsConsensus.Bytes,
			})
		case di.IsSeal:
			consensusEngineID := make([]byte, 4)
			binary.LittleEndian.PutUint32(consensusEngineID, uint32(di.AsSeal.ConsensusEngineID))
			digestItems = append(digestItems, contractsv1.VerificationDigestItem{
				Kind:              big.NewInt(5),
				ConsensusEngineID: *(*[4]byte)(consensusEngineID),
				Data:              di.AsSeal.Bytes,
			})
		default:
			return nil, fmt.Errorf("Unsupported digest item: %v", di)
		}
	}

	return &contractsv1.VerificationParachainHeader{
		ParentHash:     header.ParentHash,
		Number:         big.NewInt(int64(header.Number)),
		StateRoot:      header.StateRoot,
		ExtrinsicsRoot: header.ExtrinsicsRoot,
		DigestItems:    digestItems,
	}, nil
}
