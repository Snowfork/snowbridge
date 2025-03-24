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
	headerProof contracts.ParachainVerificationProof,
	beefyProof contracts.BeefyVerificationProof,
) log.Fields {
	messageProofHexes := make([]string, len(messageProof))
	for i, proof := range messageProof {
		messageProofHexes[i] = Hex(proof[:])
	}

	digestItems := make([]log.Fields, len(headerProof.Header.DigestItems))
	for i, digestItem := range headerProof.Header.DigestItems {
		digestItems[i] = log.Fields{
			"kind":              digestItem.Kind,
			"consensusEngineID": digestItem.ConsensusEngineID,
			"data":              Hex(digestItem.Data),
		}
	}

	headProofHexes := make([]string, len(headerProof.HeadProof.Proof))
	for i, proof := range headerProof.HeadProof.Proof {
		headProofHexes[i] = Hex(proof[:])
	}

	mmrLeafProofHexes := make([]string, len(beefyProof.LeafProof))
	for i, proof := range beefyProof.LeafProof {
		mmrLeafProofHexes[i] = Hex(proof[:])
	}

	params := log.Fields{
		"message": log.Fields{
			"channelID": Hex(message.ChannelID[:]),
			"nonce":     message.Nonce,
			"command":   message.Command,
			"params":    Hex(message.Params),
		},
		"messageProof": messageProofHexes,
		"proof": log.Fields{
			"header": log.Fields{
				"parentHash":     Hex(headerProof.Header.ParentHash[:]),
				"number":         headerProof.Header.Number,
				"stateRoot":      Hex(headerProof.Header.StateRoot[:]),
				"extrinsicsRoot": Hex(headerProof.Header.ExtrinsicsRoot[:]),
				"digestItems":    digestItems,
			},
			"headProof": log.Fields{
				"pos":   headerProof.HeadProof.Pos,
				"width": headerProof.HeadProof.Width,
				"proof": headProofHexes,
			},
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
