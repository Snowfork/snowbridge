package beefy

import (
	"context"
	"encoding/hex"
	"math/big"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type CommitmentLog struct {
	BlockNumber    uint32     `json:"blockNumber"`
	ValidatorSetID uint64     `json:"validatorSetId"`
	Payload        PayloadLog `json:"payload"`
}

type PayloadLog struct {
	MmrRootHash string `json:"mmrRootHash"`
	Prefix      string `json:"prefix"`
	Suffix      string `json:"suffix"`
}

type ProofLog struct {
	Signatures            []string         `json:"signatures"`
	Positions             []*big.Int       `json:"positions"`
	PublicKeys            []common.Address `json:"publicKeys"`
	PublicKeyMerkleProofs [][]string       `json:"publicKeyMerkleProofs"`
}

type MMRLeafLog struct {
	Version              uint8  `json:"version"`
	ParentNumber         uint32 `json:"parentNumber"`
	ParentHash           string `json:"parentHash"`
	NextAuthoritySetID   uint64 `json:"nextAuthoritySetId"`
	NextAuthoritySetLen  uint32 `json:"nextAuthoritySetLen"`
	NextAuthoritySetRoot string `json:"nextAuthoritySetRoot"`
	ParachainHeadsRoot   string `json:"parachainHeadsRoot"`
}

type MMRProofLog struct {
	MerkleProofItems []string `json:"merkleProofItems"`
	MerkleProofOrder uint64   `json:"merkleProofOrder"`
}

type FinalSignatureCommitmentLog struct {
	ID             *big.Int      `json:"id"`
	CommitmentHash string        `json:"commitmentHash"`
	Commitment     CommitmentLog `json:"commitment"`
	Proof          ProofLog      `json:"proof"`
}

type LeafUpdateLog struct {
	Leaf  MMRLeafLog  `json:"leaf"`
	Proof MMRProofLog `json:"proof"`
}

func Hex(b []byte) string {
	return gsrpcTypes.HexEncodeToString(b)
}

func (wr *EthereumWriter) LogFinal(
	task *Task,
	msg *FinalSignatureCommitment,
) error {
	var signatures []string
	for _, item := range msg.Signatures {
		signatures = append(signatures, Hex(item))
	}

	var pubKeyMerkleProofs [][]string
	for _, pubkeyProof := range msg.ValidatorPublicKeyMerkleProofs {
		var pubkeyProofS []string
		for _, item := range pubkeyProof {
			pubkeyProofS = append(pubkeyProofS, Hex(item[:]))
		}
		pubKeyMerkleProofs = append(pubKeyMerkleProofs, pubkeyProofS)
	}

	encodedCommitment, err := gsrpcTypes.EncodeToBytes(task.SignedCommitment.Commitment)
	if err != nil {
		return err
	}
	commitmentHash := Hex((&keccak.Keccak256{}).Hash(encodedCommitment))

	state := log.Fields{
		"finalSignatureCommitment": log.Fields{
			"id": msg.ID,
			"commitment": log.Fields{
				"blockNumber":    msg.Commitment.BlockNumber,
				"validatorSetId": msg.Commitment.ValidatorSetId,
				"payload": log.Fields{
					"mmrRootHash": Hex(msg.Commitment.Payload.MmrRootHash[:]),
					"prefix":      Hex(msg.Commitment.Payload.Prefix),
					"suffix":      Hex(msg.Commitment.Payload.Suffix),
				},
			},
			"proof": log.Fields{
				"signatures":            signatures,
				"positions":             msg.ValidatorPositions,
				"publicKeys":            msg.ValidatorPublicKeys,
				"publicKeyMerkleProofs": pubKeyMerkleProofs,
			},
		},
		"commitmentHash": commitmentHash,
	}

	log.WithFields(state).Debug("State for final signature commitment")

	return nil
}

func (wr *EthereumWriter) GetFailingMessage(client ethclient.Client, hash common.Hash) (string, error) {
	tx, _, err := client.TransactionByHash(context.Background(), hash)
	if err != nil {
		return "", err
	}

	from, err := types.Sender(types.NewEIP155Signer(tx.ChainId()), tx)
	if err != nil {
		return "", err
	}

	msg := ethereum.CallMsg{
		From:     from,
		To:       tx.To(),
		Gas:      tx.Gas(),
		GasPrice: tx.GasPrice(),
		Value:    tx.Value(),
		Data:     tx.Data(),
	}

	log.WithFields(logrus.Fields{
		"From":     from,
		"To":       tx.To(),
		"Gas":      tx.Gas(),
		"GasPrice": tx.GasPrice(),
		"Value":    tx.Value(),
		"Data":     hex.EncodeToString(tx.Data()),
	}).Info("Call info")

	// The logger does a test call to the actual contract to check for any revert message and log it, as well
	// as logging the call info. This is because the golang client can sometimes supress the log message and so
	// it can be helpful to use the call info to do the same call in Truffle/Web3js to get better logs.
	res, err := client.CallContract(context.Background(), msg, nil)
	if err != nil {
		return "", err
	}

	return string(res), nil
}

func (wr *EthereumWriter) LogLeafUpdate(
	task Task,
	msg *LeafUpdate,
) error {

	encodedLeaf, err := gsrpcTypes.EncodeToBytes(msg.Leaf)
	if err != nil {
		return err
	}

	leafHash := Hex((&keccak.Keccak256{}).Hash(encodedLeaf))

	var proofItems []string
	for _, item := range msg.Proof.MerkleProofItems {
		proofItems = append(proofItems, Hex(item[:]))
	}

	var leafHash2 gsrpcTypes.H256
	copy(leafHash2[:], (&keccak.Keccak256{}).Hash(encodedLeaf))

	root := merkle.CalculateMerkleRoot(&task.Proof, leafHash2)

	state := log.Fields{
		"updateLeaf": log.Fields{
			"leaf": log.Fields{
				"version":              msg.Leaf.Version,
				"parentNumber":         msg.Leaf.ParentNumber,
				"parentHash":           Hex(msg.Leaf.ParentHash[:]),
				"nextAuthoritySetId":   msg.Leaf.NextAuthoritySetId,
				"nextAuthoritySetLen":  msg.Leaf.NextAuthoritySetLen,
				"nextAuthoritySetRoot": Hex(msg.Leaf.NextAuthoritySetRoot[:]),
				"parachainHeadsRoot":   Hex(msg.Leaf.ParachainHeadsRoot[:]),
			},
			"proof": log.Fields{
				"merkleProofItems":         proofItems,
				"merkleProofOrderBitField": msg.Proof.MerkleProofOrderBitField,
			},
		},
		"encodedLeaf":     Hex(encodedLeaf),
		"leafHash":        leafHash,
		"expectedMMRRoot": root.Hex(),
	}

	log.WithFields(state).Debug("State for update validator set")

	return nil
}
