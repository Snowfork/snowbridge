package beefy

import (
	log "github.com/sirupsen/logrus"
	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
)

func Hex(b []byte) string {
	return gsrpcTypes.HexEncodeToString(b)
}

func (wr *EthereumWriter) makeSubmitFinalLogFields(
	task *Request,
	params *FinalRequestParams,
) (log.Fields, error) {
	var signatures []string
	for _, item := range params.Proof.Signatures {
		signatures = append(signatures, Hex(item))
	}

	var merkleProofs [][]string
	for _, merkleProof := range params.Proof.MerkleProofs {
		var acc []string
		for _, item := range merkleProof {
			acc = append(acc, Hex(item[:]))
		}
		merkleProofs = append(merkleProofs, acc)
	}

	encodedCommitment, err := gsrpcTypes.EncodeToBytes(task.SignedCommitment.Commitment)
	if err != nil {
		return nil, err
	}
	commitmentHash := Hex((&keccak.Keccak256{}).Hash(encodedCommitment))

	fields := log.Fields{
		"params": log.Fields{
			"id": params.ID,
			"commitment": log.Fields{
				"blockNumber":    params.Commitment.BlockNumber,
				"validatorSetID": params.Commitment.ValidatorSetID,
				"payload": log.Fields{
					"mmrRootHash": Hex(params.Commitment.Payload.MmrRootHash[:]),
					"prefix":      Hex(params.Commitment.Payload.Prefix),
					"suffix":      Hex(params.Commitment.Payload.Suffix),
				},
			},
			"proof": log.Fields{
				"signatures":   signatures,
				"indices":      params.Proof.Indices,
				"addrs":        params.Proof.Addrs,
				"merkleProofs": merkleProofs,
			},
		},
		"commitmentHash": commitmentHash,
	}

	return fields, nil
}

func (wr *EthereumWriter) makeSubmitFinalHandoverLogFields(
	task *Request,
	params *FinalRequestParams,
) (log.Fields, error) {
	var signatures []string
	for _, item := range params.Proof.Signatures {
		signatures = append(signatures, Hex(item))
	}

	var merkleProofs [][]string
	for _, merkleProof := range params.Proof.MerkleProofs {
		var acc []string
		for _, item := range merkleProof {
			acc = append(acc, Hex(item[:]))
		}
		merkleProofs = append(merkleProofs, acc)
	}

	encodedCommitment, err := gsrpcTypes.EncodeToBytes(task.SignedCommitment.Commitment)
	if err != nil {
		return nil, err
	}
	commitmentHash := Hex((&keccak.Keccak256{}).Hash(encodedCommitment))

	var proofItems []string
	for _, item := range params.LeafProof.Items {
		proofItems = append(proofItems, Hex(item[:]))
	}

	fields := log.Fields{
		"params": log.Fields{
			"id": params.ID,
			"commitment": log.Fields{
				"blockNumber":    params.Commitment.BlockNumber,
				"validatorSetID": params.Commitment.ValidatorSetID,
				"payload": log.Fields{
					"mmrRootHash": Hex(params.Commitment.Payload.MmrRootHash[:]),
					"prefix":      Hex(params.Commitment.Payload.Prefix),
					"suffix":      Hex(params.Commitment.Payload.Suffix),
				},
			},
			"proof": log.Fields{
				"signatures":   signatures,
				"indices":      params.Proof.Indices,
				"addrs":        params.Proof.Addrs,
				"merkleProofs": merkleProofs,
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
				"items": proofItems,
				"order": params.LeafProof.Order,
			},
		},
		"commitmentHash": commitmentHash,
	}

	return fields, nil
}
