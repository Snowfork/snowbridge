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
	var proofs []log.Fields
	for _, proof := range params.Proofs {
		var merkleProof []string
		for _, item := range proof.Proof {
			merkleProof = append(merkleProof, Hex(item[:]))
		}
		proofs = append(proofs,
			log.Fields{
				"v":       proof.V,
				"r":       Hex(proof.R[:]),
				"s":       Hex(proof.S[:]),
				"index":   proof.Index.Uint64(),
				"account": proof.Account.Hex(),
				"proof":   merkleProof,
			},
		)
	}

	encodedCommitment, err := gsrpcTypes.EncodeToBytes(task.SignedCommitment.Commitment)
	if err != nil {
		return nil, err
	}
	commitmentHash := Hex((&keccak.Keccak256{}).Hash(encodedCommitment))

	fields := log.Fields{
		"params": log.Fields{
			"commitment": log.Fields{
				"blockNumber":    params.Commitment.BlockNumber,
				"validatorSetID": params.Commitment.ValidatorSetID,
				"payload": log.Fields{
					"mmrRootHash": Hex(params.Commitment.Payload.MmrRootHash[:]),
				},
			},
			"bitfield": params.Bitfield,
			"proof":    proofs,
		},
		"commitmentHash": commitmentHash,
	}

	return fields, nil
}

func (wr *EthereumWriter) makeSubmitFinalHandoverLogFields(
	task *Request,
	params *FinalRequestParams,
) (log.Fields, error) {
	var proofs []log.Fields
	for _, proof := range params.Proofs {
		var merkleProof []string
		for _, item := range proof.Proof {
			merkleProof = append(merkleProof, Hex(item[:]))
		}
		proofs = append(proofs,
			log.Fields{
				"v":       proof.V,
				"r":       Hex(proof.R[:]),
				"s":       Hex(proof.S[:]),
				"index":   proof.Index,
				"account": proof.Account.Hex(),
				"proof":   merkleProof,
			},
		)
	}

	encodedCommitment, err := gsrpcTypes.EncodeToBytes(task.SignedCommitment.Commitment)
	if err != nil {
		return nil, err
	}
	commitmentHash := Hex((&keccak.Keccak256{}).Hash(encodedCommitment))

	var leafProofItems []string
	for _, item := range params.LeafProof {
		leafProofItems = append(leafProofItems, Hex(item[:]))
	}

	fields := log.Fields{
		"params": log.Fields{
			"commitment": log.Fields{
				"blockNumber":    params.Commitment.BlockNumber,
				"validatorSetID": params.Commitment.ValidatorSetID,
				"payload": log.Fields{
					"mmrRootHash": Hex(params.Commitment.Payload.MmrRootHash[:]),
				},
			},
			"bitfield": params.Bitfield,
			"proofs":   proofs,
			"leaf": log.Fields{
				"version":              params.Leaf.Version,
				"parentNumber":         params.Leaf.ParentNumber,
				"parentHash":           Hex(params.Leaf.ParentHash[:]),
				"nextAuthoritySetID":   params.Leaf.NextAuthoritySetID,
				"nextAuthoritySetLen":  params.Leaf.NextAuthoritySetLen,
				"nextAuthoritySetRoot": Hex(params.Leaf.NextAuthoritySetRoot[:]),
				"parachainHeadsRoot":   Hex(params.Leaf.ParachainHeadsRoot[:]),
			},
			"leafProof":      leafProofItems,
			"leafProofOrder": params.LeafProofOrder,
		},
		"commitmentHash": commitmentHash,
	}

	return fields, nil
}
