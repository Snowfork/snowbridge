package beefy

import (
	"context"
	"encoding/hex"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
)

func Hex(b []byte) string {
	return gsrpcTypes.HexEncodeToString(b)
}

func (wr *EthereumWriter) LogFinal(
	task *Request,
	params *FinalRequestParams,
) error {
	var signatures []string
	for _, item := range params.Proof.Signatures {
		signatures = append(signatures, Hex(item))
	}

	var pubKeyMerkleProofs [][]string
	for _, pubkeyProof := range params.Proof.MerkleProofs {
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

	var state log.Fields

	var proofItems []string
	for _, item := range params.LeafProof.Items {
		proofItems = append(proofItems, Hex(item[:]))
	}

	state = log.Fields{
		"transactionParams": log.Fields{
			"id": params.ID,
			"commitment": log.Fields{
				"blockNumber":    params.Commitment.BlockNumber,
				"validatorSetId": params.Commitment.ValidatorSetID,
				"payload": log.Fields{
					"mmrRootHash": Hex(params.Commitment.Payload.MmrRootHash[:]),
					"prefix":      Hex(params.Commitment.Payload.Prefix),
					"suffix":      Hex(params.Commitment.Payload.Suffix),
				},
			},
			"proof": log.Fields{
				"signatures":            signatures,
				"indices":             params.Proof.Indices,
				"addrs":            params.Proof.Addrs,
				"merkleProofs": pubKeyMerkleProofs,
			},
			"leaf": log.Fields{
				"version":              params.Leaf.Version,
				"parentNumber":         params.Leaf.ParentNumber,
				"parentHash":           Hex(params.Leaf.ParentHash[:]),
				"nextAuthoritySetID":   params.Leaf.NextAuthoritySetID,
				"nextAuthoritySetLen":  params.Leaf.NextAuthoritySetLen,
				"nextAuthoritySetRoot": Hex(params.Leaf.NextAuthoritySetRoot[:]),
				"parachainHeadsRoot":   Hex(params.Leaf.ParachainHeadsRoot[:]),
			},
			"leafProof": log.Fields{
				"Items": proofItems,
				"Order": params.LeafProof.Order,
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

	params := ethereum.CallMsg{
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
	res, err := client.CallContract(context.Background(), params, nil)
	if err != nil {
		return "", err
	}

	return string(res), nil
}
