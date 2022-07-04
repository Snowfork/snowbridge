package beacon

import (
	"context"
	"path/filepath"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
	"github.com/snowfork/snowbridge/relayer/relays/ethereum/syncer"
	"golang.org/x/sync/errgroup"
)

type ParachainPayload struct {
	Messages []*chain.EthereumOutboundMessage
}

type EthereumListener struct {
	ethashDataDir               string
	ethashCacheDir              string
	config                      *SourceConfig
	conn                        *ethereum.Connection
	basicOutboundChannel        *basic.BasicOutboundChannel
	incentivizedOutboundChannel *incentivized.IncentivizedOutboundChannel
	mapping                     map[common.Address]string
	payloads                    chan ParachainPayload
	headerSyncer                *syncer.Syncer
	headerCache                 *ethereum.HeaderCache
}

func NewEthereumListener(
	config *SourceConfig,
	conn *ethereum.Connection,
) *EthereumListener {
	return &EthereumListener{
		ethashDataDir:               filepath.Join(config.DataDir, "ethash-data"),
		ethashCacheDir:              filepath.Join(config.DataDir, "ethash-cache"),
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

	li.headerCache, err = ethereum.NewHeaderCacheWithBlockCacheOnly(
		&ethereum.DefaultBlockLoader{Conn: li.conn},
	)
	if err != nil {
		return err
	}

	var address common.Address

	address = common.HexToAddress(li.config.Contracts.BasicOutboundChannel)
	basicOutboundChannel, err := basic.NewBasicOutboundChannel(address, li.conn.Client())
	log.WithField("basicOutboundChannel", basicOutboundChannel).Info("log basic outbound channel")
	if err != nil {
		return err
	}

	li.basicOutboundChannel = basicOutboundChannel
	li.mapping[address] = "BasicInboundChannel.submit"

	return nil
}

func (li *EthereumListener) ProcessEvents(
	ctx context.Context,
	start uint64,
	end uint64,
) (ParachainPayload, error) {
	log.Info("Syncing events starting...")

	var events []*etypes.Log

	//filterOptions := bind.FilterOpts{Context: ctx}
	filterOptions := bind.FilterOpts{Start: start, End: &end, Context: ctx}
	log.WithField("li.basicOutboundChannel", li.basicOutboundChannel).Info("log basic outbound channel")
	basicEvents, err := li.queryBasicEvents(li.basicOutboundChannel, &filterOptions)
	log.WithField("basicEvents", basicEvents).Info("log basic events")
	if err != nil {
		log.WithError(err).Error("Failure fetching event logs")
		return ParachainPayload{}, err
	}
	events = append(events, basicEvents...)

	messages, err := li.makeOutgoingMessages(ctx, li.headerCache, events)
	if err != nil {
		return ParachainPayload{}, err
	}

	return ParachainPayload{Messages: messages}, nil
}

func (li *EthereumListener) queryBasicEvents(contract *basic.BasicOutboundChannel, options *bind.FilterOpts) ([]*etypes.Log, error) {
	var events []*etypes.Log

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
		events = append(events, &iter.Event.Raw)
	}
	return events, nil
}

func (li *EthereumListener) makeOutgoingMessages(
	ctx context.Context,
	hcs *ethereum.HeaderCache,
	events []*etypes.Log,
) ([]*chain.EthereumOutboundMessage, error) {
	messages := make([]*chain.EthereumOutboundMessage, len(events))

	for i, event := range events {
		receiptTrie, err := hcs.GetReceiptTrie(ctx, event.BlockHash)
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

		log.WithField("message", msg).Info("Got message")

		messages[i] = msg
	}

	return messages, nil
}
