package parachain

import (
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
)

func Hex(b []byte) string {
	return types.HexEncodeToString(b)
}

func (wr *EthereumWriter) logFieldsForBasicSubmission(
	bundle basic.BasicInboundChannelMessageBundle,
	leafProof [][32]byte,
	hashSides []bool,
	proof []byte,
) log.Fields {
	var messagesLog []log.Fields
	for _, item := range bundle.Messages {
		messagesLog = append(messagesLog, log.Fields{
			"id":      item.Id,
			"target":  item.Target,
			"payload": Hex(item.Payload),
		})
	}

	leafProofHexes := make([]string, len(leafProof))
	for i, leaf := range leafProof {
		leafProofHexes[i] = Hex(leaf[:])
	}

	params := log.Fields{
		"bundle": log.Fields{
			"sourceChannelID": bundle.SourceChannelID,
			"nonce":           bundle.Nonce,
			"account":         Hex(bundle.Account[:]),
			"messages":        messagesLog,
		},
		"proof":     Hex(proof),
		"leafProof": leafProofHexes,
		"hashSides": hashSides,
	}

	return params
}
