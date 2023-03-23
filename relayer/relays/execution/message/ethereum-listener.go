package message

import (
	"context"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/execution/config"
	"golang.org/x/sync/errgroup"
)

type ParachainPayload struct {
	Messages []*chain.EthereumOutboundMessage
}

type EventContainer struct {
	Event *etypes.Log
	Dest  []byte
	Nonce uint64
}

type EthereumListener struct {
	config          *config.SourceConfig
	conn            *ethereum.Connection
	outboundChannel *contracts.OutboundChannel
	headerCache     *ethereum.HeaderCache
}

func NewEthereumListener(
	config *config.SourceConfig,
	conn *ethereum.Connection,
) *EthereumListener {
	return &EthereumListener{
		config:          config,
		conn:            conn,
		outboundChannel: nil,
	}
}

func (li *EthereumListener) Start(
	ctx context.Context,
	eg *errgroup.Group,
) error {
	var err error

	li.headerCache, err = ethereum.NewHeaderBlockCache(
		&ethereum.DefaultBlockLoader{Conn: li.conn},
	)
	if err != nil {
		return err
	}

	address := common.HexToAddress(li.config.Contracts.OutboundChannel)
	outboundChannel, err := contracts.NewOutboundChannel(address, li.conn.Client())
	if err != nil {
		return err
	}
	li.outboundChannel = outboundChannel

	return nil
}

func (li *EthereumListener) ProcessEvents(
	ctx context.Context,
	start uint64,
	end uint64,
	addressNonceMap map[common.Address]uint64,
) (ParachainPayload, error) {
	filterOptions := bind.FilterOpts{Start: start, End: &end, Context: ctx}
	basicEvents, err := li.queryEvents(li.outboundChannel, addressNonceMap, &filterOptions)
	if err != nil {
		return ParachainPayload{}, err
	}

	messages, err := li.makeOutgoingMessages(ctx, basicEvents)
	if err != nil {
		return ParachainPayload{}, err
	}

	return ParachainPayload{Messages: messages}, nil
}

func (li *EthereumListener) queryEvents(contract *contracts.OutboundChannel, addressNonceMap map[common.Address]uint64, options *bind.FilterOpts) ([]EventContainer, error) {
	var events []EventContainer

	iter, err := contract.FilterMessage(options)
	if err != nil {
		return nil, err
	}

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			if err != nil {
				return nil, err
			}
			break
		}

		events = append(events, EventContainer{
			Event: &iter.Event.Raw,
			Dest:  iter.Event.Dest,
			Nonce: iter.Event.Nonce,
		})
	}
	return events, nil
}

func (li *EthereumListener) makeOutgoingMessages(
	ctx context.Context,
	events []EventContainer,
) ([]*chain.EthereumOutboundMessage, error) {
	messages := make([]*chain.EthereumOutboundMessage, len(events))

	for i, eventContainer := range events {
		event := eventContainer.Event
		receiptTrie, err := li.headerCache.GetReceiptTrie(ctx, event.BlockHash)
		if err != nil {
			log.WithFields(logrus.Fields{
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to get receipt trie for event")
			return nil, err
		}

		msg, err := ethereum.MakeMessageFromEvent(event, receiptTrie)
		if err != nil {
			log.WithFields(logrus.Fields{
				"address":     event.Address.Hex(),
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to generate message from ethereum event")
			return nil, err
		}

		msg.Origin = eventContainer.Origin
		msg.Nonce = eventContainer.Nonce

		messages[i] = msg
	}

	return messages, nil
}
