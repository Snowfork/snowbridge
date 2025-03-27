package parachain

import (
	"fmt"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts"
)

func Hex(b []byte) string {
	return types.HexEncodeToString(b)
}

func (wr *EthereumWriter) logFieldsForSubmission(
	message contracts.InboundMessage,
	messageProof [][32]byte,
	beefyProof contracts.BeefyVerificationProof,
) log.Fields {
	messageProofHexes := make([]string, len(messageProof))
	for i, proof := range messageProof {
		messageProofHexes[i] = Hex(proof[:])
	}

	mmrLeafProofHexes := make([]string, len(beefyProof.LeafProof))
	for i, proof := range beefyProof.LeafProof {
		mmrLeafProofHexes[i] = Hex(proof[:])
	}

	params := log.Fields{
		"message": log.Fields{
			"nonce":    message.Nonce,
			"commands": message.Commands,
			"origin":   Hex(message.Origin[:]),
		},
		"messageProof": messageProofHexes,
		"proof": log.Fields{
			"leafPartial": log.Fields{
				"version":              beefyProof.LeafPartial.Version,
				"parentNumber":         beefyProof.LeafPartial.ParentNumber,
				"parentHash":           Hex(beefyProof.LeafPartial.ParentHash[:]),
				"nextAuthoritySetID":   beefyProof.LeafPartial.NextAuthoritySetID,
				"nextAuthoritySetLen":  beefyProof.LeafPartial.NextAuthoritySetLen,
				"nextAuthoritySetRoot": Hex(beefyProof.LeafPartial.NextAuthoritySetRoot[:]),
			},
			"leafProof":      mmrLeafProofHexes,
			"leafProofOrder": fmt.Sprintf("%b", beefyProof.LeafProofOrder),
		},
	}

	return params
}
