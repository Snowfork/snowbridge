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

type ValidatorProofLog struct {
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
	ID                 *big.Int          `json:"id"`
	CommitmentHash     string            `json:"commitmentHash"`
	Commitment         CommitmentLog     `json:"commitment"`
	ValidatorProof     ValidatorProofLog `json:"validatorProof"`
	LatestMMRLeaf      MMRLeafLog        `json:"latestMMRLeaf"`
	SimplifiedMMRProof MMRProofLog        `json:"simplifiedMMRProof"`
}

func Hex(b []byte) string {
	return gsrpcTypes.HexEncodeToString(b)
}

func (wr *EthereumWriter) LogBeefyFixtureDataAll(
	task *Task,
	msg *FinalSignatureCommitment,
) error {

	encodedLeaf, err := gsrpcTypes.EncodeToBytes(msg.LatestMMRLeaf)
	if err != nil {
		return err
	}

	leafHash := Hex((&keccak.Keccak256{}).Hash(encodedLeaf))

	var beefyMMRMerkleProofItems []string
	for _, item := range msg.SimplifiedProof.MerkleProofItems {
		beefyMMRMerkleProofItems = append(beefyMMRMerkleProofItems, Hex(item[:]))
	}

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
				"blockNumber": msg.Commitment.BlockNumber,
				"validatorSetId": msg.Commitment.ValidatorSetId,
				"payload": log.Fields{
					"mmrRootHash": Hex(msg.Commitment.Payload.MmrRootHash[:]),
					"prefix": Hex(msg.Commitment.Payload.Prefix),
					"suffix": Hex(msg.Commitment.Payload.Suffix),
				},
			},
			"validatorProof": log.Fields{
				"signatures": signatures,
				"positions": msg.ValidatorPositions,
				"publicKeys": msg.ValidatorPublicKeys,
				"publicKeyMerkleProofs": pubKeyMerkleProofs,
			},
			"leaf": log.Fields{
				"version": msg.LatestMMRLeaf.Version,
				"parentNumber": msg.LatestMMRLeaf.ParentNumber,
				"parentHash": Hex(msg.LatestMMRLeaf.ParentHash[:]),
				"nextAuthoritySetId": msg.LatestMMRLeaf.NextAuthoritySetId,
				"nextAuthoritySetLen": msg.LatestMMRLeaf.NextAuthoritySetLen,
				"nextAuthoritySetRoot": Hex(msg.LatestMMRLeaf.NextAuthoritySetRoot[:]),
				"parachainHeadsRoot": Hex(msg.LatestMMRLeaf.ParachainHeadsRoot[:]),
			},
			"proof": log.Fields{
				"merkleProofItems": beefyMMRMerkleProofItems,
				"merkleProofOrderBitField": msg.SimplifiedProof.MerkleProofOrderBitField,
			},
		},
		"commitmentHash": commitmentHash,
		"encodedLeaf": Hex(encodedLeaf),
		"leafHash": leafHash,
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
