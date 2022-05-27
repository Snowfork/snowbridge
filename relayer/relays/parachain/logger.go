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

	params := log.Fields{
		"bundle": log.Fields{
			"sourceChannelID": bundle.SourceChannelID,
			"nonce":    bundle.Nonce,
			"messages": messagesLog,
		},
		"proof": Hex(proof),
	}

	return params
}

func (wr *EthereumWriter) logFieldsForIncentivizedSubmission(
	bundle incentivized.IncentivizedInboundChannelMessageBundle,
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

	params := log.Fields{
		"bundle": log.Fields{
			"sourceChannelID": bundle.SourceChannelID,
			"nonce":    bundle.Nonce,
			"fee": bundle.Fee.String(),
			"messages": messagesLog,
		},
		"proof": Hex(proof),
	}

	return params
}
