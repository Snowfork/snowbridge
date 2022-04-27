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
	messages []basic.BasicInboundChannelMessage,
	proof basic.ParachainClientProof,
) log.Fields {
	var messagesLog []log.Fields
	for _, item := range messages {
		messagesLog = append(messagesLog, log.Fields{
			"target":  item.Target,
			"nonce":   item.Nonce,
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
		"messages": messagesLog,
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
				"nextAuthoritySetID":   proof.LeafPartial.NextAuthoritySetId,
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
	messages []incentivized.IncentivizedInboundChannelMessage,
	proof incentivized.ParachainClientProof,
) log.Fields {
	var messagesLog []log.Fields
	for _, item := range messages {
		messagesLog = append(messagesLog, log.Fields{
			"target":  item.Target,
			"nonce":   item.Nonce,
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
		"messages": messagesLog,
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
				"nextAuthoritySetID":   proof.LeafPartial.NextAuthoritySetId,
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
