package parachain

import (
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
)

func Hex(b []byte) string {
	return types.HexEncodeToString(b)
}

func (wr *EthereumWriter) logFieldsForBasicSubmission(
	bundle basic.BasicInboundChannelMessageBundle,
	proof basic.ParachainClientProof,
) log.Fields {
	var messagesLog []log.Fields
	for _, item := range bundle.Messages {
		messagesLog = append(messagesLog, log.Fields{
			"id":      item.Id,
			"target":  item.Target,
			"payload": Hex(item.Payload),
		})
	}
	var paraHeadProofString []string
	for _, item := range proof.HeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, Hex(item[:]))
	}

	var mmrLeafProofItems []string
	for _, item := range proof.LeafProof.Items {
		mmrLeafProofItems = append(mmrLeafProofItems, Hex(item[:]))
	}

	params := log.Fields{
		"bundle": log.Fields{
			"nonce":    bundle.Nonce,
			"messages": messagesLog,
		},
		"proof": log.Fields{
			"headPrefix": Hex(proof.HeadPrefix),
			"headSuffix": Hex(proof.HeadSuffix),
			"headProof": log.Fields{
				"pos":   proof.HeadProof.Pos,
				"width": proof.HeadProof.Width,
				"proof": paraHeadProofString,
			},
			"leafPartial": log.Fields{
				"version":              proof.LeafPartial.Version,
				"parentNumber":         proof.LeafPartial.ParentNumber,
				"parentHash":           Hex(proof.LeafPartial.ParentHash[:]),
				"nextAuthoritySetID":   proof.LeafPartial.NextAuthoritySetID,
				"nextAuthoritySetLen":  proof.LeafPartial.NextAuthoritySetLen,
				"nextAuthoritySetRoot": Hex(proof.LeafPartial.NextAuthoritySetRoot[:]),
			},
			"leafProof": log.Fields{
				"items": mmrLeafProofItems,
				"order": proof.LeafProof.Order,
			},
		},
	}

	return params
}

func (wr *EthereumWriter) logFieldsForIncentivizedSubmission(
	bundle incentivized.IncentivizedInboundChannelMessageBundle,
	proof incentivized.ParachainClientProof,
) log.Fields {
	var messagesLog []log.Fields
	for _, item := range bundle.Messages {
		messagesLog = append(messagesLog, log.Fields{
			"id":      item.Id,
			"target":  item.Target,
			"fee":     item.Fee,
			"payload": Hex(item.Payload),
		})
	}
	var paraHeadProofString []string
	for _, item := range proof.HeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, Hex(item[:]))
	}

	var mmrLeafProofItems []string
	for _, item := range proof.LeafProof.Items {
		mmrLeafProofItems = append(mmrLeafProofItems, Hex(item[:]))
	}

	params := log.Fields{
		"bundle": log.Fields{
			"nonce":    bundle.Nonce,
			"messages": messagesLog,
		},
		"proof": log.Fields{
			"headPrefix": Hex(proof.HeadPrefix),
			"headSuffix": Hex(proof.HeadSuffix),
			"headProof": log.Fields{
				"pos":   proof.HeadProof.Pos,
				"width": proof.HeadProof.Width,
				"proof": paraHeadProofString,
			},
			"leafPartial": log.Fields{
				"version":              proof.LeafPartial.Version,
				"parentNumber":         proof.LeafPartial.ParentNumber,
				"parentHash":           Hex(proof.LeafPartial.ParentHash[:]),
				"nextAuthoritySetID":   proof.LeafPartial.NextAuthoritySetID,
				"nextAuthoritySetLen":  proof.LeafPartial.NextAuthoritySetLen,
				"nextAuthoritySetRoot": Hex(proof.LeafPartial.NextAuthoritySetRoot[:]),
			},
			"leafProof": log.Fields{
				"items": mmrLeafProofItems,
				"order": proof.LeafProof.Order,
			},
		},
	}

	return params
}
