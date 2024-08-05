package beefy

import (
	log "github.com/sirupsen/logrus"
	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

func Hex(b []byte) string {
	return gsrpcTypes.HexEncodeToString(b)
}

func (wr *EthereumWriter) makeSubmitFinalLogFields(
	task *Request,
	params *FinalRequestParams,
) (log.Fields, error) {
	proofs := make([]log.Fields, len(params.Proofs))
	for i, proof := range params.Proofs {
		proofs[i] = proofToLog(proof)
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
			"commitment": commitmentToLog(params.Commitment),
			"bitfield":   bitfieldToStrings(params.Bitfield),
			"proofs":     proofs,
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

func (wr *EthereumWriter) generateSubmitInitial(
	task *InitialRequestParams,
) error {
	jsonTask := task.ToJSON()
	return util.WriteJSONToFile(jsonTask, "contracts/test/data/initial-commitment.json")
}

func (wr *EthereumWriter) generateSubmitFinal(
	task *FinalRequestParams,
) error {
	return util.WriteJSONToFile(*task, "contracts/test/data/final-commitment.json")
}
