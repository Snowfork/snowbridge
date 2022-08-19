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
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/ethereum/syncer"
	"golang.org/x/sync/errgroup"
)

type ParachainPayload struct {
	Messages []*chain.EthereumOutboundMessage
}

type EventContainer struct {
	Event *etypes.Log
	Nonce uint64
}

type EthereumListener struct {
	config                      *config.SourceConfig
	conn                        *ethereum.Connection
	basicOutboundChannel        *basic.BasicOutboundChannel
	incentivizedOutboundChannel *incentivized.IncentivizedOutboundChannel
	mapping                     map[common.Address]string
	headerSyncer                *syncer.Syncer
	headerCache                 *ethereum.HeaderCache
}

func NewEthereumListener(
	config *config.SourceConfig,
	conn *ethereum.Connection,
) *EthereumListener {
	return &EthereumListener{
		config:                      config,
		conn:                        conn,
		basicOutboundChannel:        nil,
		incentivizedOutboundChannel: nil,
		mapping:                     make(map[common.Address]string),
		headerSyncer:                nil,
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

	var address common.Address

	address = common.HexToAddress(li.config.Contracts.BasicOutboundChannel)
	basicOutboundChannel, err := basic.NewBasicOutboundChannel(address, li.conn.Client())
	if err != nil {
		return err
	}
	li.basicOutboundChannel = basicOutboundChannel
	li.mapping[address] = "BasicInboundChannel.submit"

	address = common.HexToAddress(li.config.Contracts.IncentivizedOutboundChannel)
	incentivizedOutboundChannel, err := incentivized.NewIncentivizedOutboundChannel(address, li.conn.Client())
	if err != nil {
		return err
	}
	li.incentivizedOutboundChannel = incentivizedOutboundChannel
	li.mapping[address] = "IncentivizedInboundChannel.submit"

	return nil
}

func (li *EthereumListener) ProcessBasicEvents(
	ctx context.Context,
	start uint64,
	end uint64,
) (ParachainPayload, error) {
	filterOptions := bind.FilterOpts{Start: start, End: &end, Context: ctx}
	basicEvents, err := li.queryBasicEvents(li.basicOutboundChannel, &filterOptions)
	if err != nil {
		return ParachainPayload{}, err
	}

	messages, err := li.makeOutgoingMessages(ctx, basicEvents)
	if err != nil {
		return ParachainPayload{}, err
	}

	return ParachainPayload{Messages: messages}, nil
}

func (li *EthereumListener) ProcessIncentivizedEvents(
	ctx context.Context,
	start uint64,
	end uint64,
) (ParachainPayload, error) {
	filterOptions := bind.FilterOpts{Start: start, End: &end, Context: ctx}

	incentivizedEvents, err := li.queryIncentivizedEvents(li.incentivizedOutboundChannel, &filterOptions)
	if err != nil {
		return ParachainPayload{}, err
	}

	messages, err := li.makeOutgoingMessages(ctx, incentivizedEvents)
	if err != nil {
		return ParachainPayload{}, err
	}

	return ParachainPayload{Messages: messages}, nil
}

func (li *EthereumListener) queryBasicEvents(contract *basic.BasicOutboundChannel, options *bind.FilterOpts) ([]EventContainer, error) {
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
			Nonce: iter.Event.Nonce,
		})
	}
	return events, nil
}

func (li *EthereumListener) queryIncentivizedEvents(contract *incentivized.IncentivizedOutboundChannel, options *bind.FilterOpts) ([]EventContainer, error) {
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

		msg, err := ethereum.MakeMessageFromEvent(li.mapping, event, receiptTrie)
		if err != nil {
			log.WithFields(logrus.Fields{
				"address":     event.Address.Hex(),
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to generate message from ethereum event")
			return nil, err
		}

		msg.Nonce = eventContainer.Nonce

		messages[i] = msg
	}

	return messages, nil
}
