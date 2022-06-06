package beacon

import (
	"context"
	"errors"
	"os"
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

type ContractsConfig struct {
	BasicOutboundChannel        string `mapstructure:"BasicOutboundChannel"`
	IncentivizedOutboundChannel string `mapstructure:"IncentivizedOutboundChannel"`
}

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
	initBlockHeight             uint64
}

func NewEthereumListener(
	config *SourceConfig,
	conn *ethereum.Connection,
	initBlockHeight uint64,
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
		initBlockHeight:             initBlockHeight,
	}
}

func (li *EthereumListener) Start(
	ctx context.Context,
	eg *errgroup.Group,
) (<-chan ParachainPayload, error) {
	var err error

	li.payloads = make(chan ParachainPayload, 1)

	err = os.Mkdir(li.ethashDataDir, 0755)
	if err != nil && !errors.Is(err, os.ErrExist) {
		log.WithError(err).Error("Could not create ethash data dir")
		return nil, err
	}

	err = os.Mkdir(li.ethashCacheDir, 0755)
	if err != nil && !errors.Is(err, os.ErrExist) {
		log.WithError(err).Error("Could not create ethash cache dir")
		return nil, err
	}

	headerCache, err := ethereum.NewHeaderCache(
		li.ethashDataDir,
		li.ethashCacheDir,
		eg,
		li.initBlockHeight,
		&ethereum.DefaultBlockLoader{Conn: li.conn},
		nil,
	)
	if err != nil {
		return nil, err
	}

	var address common.Address

	address = common.HexToAddress(li.config.Contracts.BasicOutboundChannel)
	basicOutboundChannel, err := basic.NewBasicOutboundChannel(address, li.conn.Client())
	if err != nil {
		return nil, err
	}
	li.basicOutboundChannel = basicOutboundChannel
	li.mapping[address] = "BasicInboundChannel.submit"

	eg.Go(func() error {
		defer close(li.payloads)
		err := li.processEventsAndHeaders(ctx, headerCache, 100, 200)
		log.WithField("reason", err).Info("Shutting down ethereum listener")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})

	return li.payloads, nil
}

func (li *EthereumListener) processEventsAndHeaders(
	ctx context.Context,
	headerCache *ethereum.HeaderCache,
	start uint64,
	end uint64,
) error {
	log.Info("Syncing headers starting...")

	for {
		var events []*etypes.Log

		filterOptions := bind.FilterOpts{Start: start, End: &end, Context: ctx}

		basicEvents, err := li.queryBasicEvents(li.basicOutboundChannel, &filterOptions)
		if err != nil {
			log.WithError(err).Error("Failure fetching event logs")
			return err
		}
		events = append(events, basicEvents...)

		messages, err := li.makeOutgoingMessages(ctx, headerCache, events)
		if err != nil {
			return err
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.payloads <- ParachainPayload{Messages: messages}:
		}
	}
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

		messages[i] = msg
	}

	return messages, nil
}
