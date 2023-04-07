package parachain

import (
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts"
)

func Hex(b []byte) string {
	return types.HexEncodeToString(b)
}

func (wr *EthereumWriter) logFieldsForBasicSubmission(
	message contracts.InboundQueueMessage,
	leafProof [][32]byte,
	hashSides []bool,
	proof []byte,
) log.Fields {
	leafProofHexes := make([]string, len(leafProof))
	for i, leaf := range leafProof {
		leafProofHexes[i] = Hex(leaf[:])
	}

	params := log.Fields{
		"message": log.Fields{
			"origin":  Hex(message.Origin),
			"nonce":   message.Nonce,
			"payload": message.Payload,
		},
		"leafProof": leafProofHexes,
		"hashSides": hashSides,
		"proof":     Hex(proof),
	}

	return params
}
