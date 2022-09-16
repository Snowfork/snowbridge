package message

import (
	"context"
	"fmt"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/writer"
	"golang.org/x/sync/errgroup"
)

type Message struct {
	writer   *writer.ParachainWriter
	listener *EthereumListener
}

func New(ctx context.Context, eg *errgroup.Group, writer *writer.ParachainWriter, configSource *config.SourceConfig, ethconn *ethereum.Connection) (*Message, error) {
	listener := NewEthereumListener(
		configSource,
		ethconn,
	)

	err := listener.Start(ctx, eg)
	if err != nil {
		return &Message{}, err
	}

	return &Message{writer, listener}, nil
}

func (m *Message) SyncBasic(ctx context.Context, eg *errgroup.Group, blockNumber <-chan uint64) error {
	lastVerifiedBlockNumber, err := m.writer.GetLastBasicChannelMessage()
	if err != nil {
		return fmt.Errorf("fetch last basic channel message block number")
	}

	nonce, err := m.writer.GetLastBasicChannelNonce()
	if err != nil {
		return fmt.Errorf("fetch last basic channel message nonce")
	}

	log.WithFields(log.Fields{
		"block_number": lastVerifiedBlockNumber,
		"nonce":        nonce,
	}).Info("last basic channel")

	// If the last nonce is set, there could be messages that have not been processed in the same block.
	// Messages that have already been verified will not be reprocessed because they will be filtered out
	// in filterMessagesByLastNonce.
	// Messages after the lastVerifiedBlockNumber will be processed normally in the go routine below.
	if nonce != 0 {
		log.Info("processing basic block events for last verified block")
		basicPayload, err := m.listener.ProcessBasicEvents(ctx, lastVerifiedBlockNumber, lastVerifiedBlockNumber)
		if err != nil {
			return err
		}

		basicPayload.Messages = filterMessagesByLastNonce(basicPayload.Messages, nonce)
		// Reset the nonce so that the next block processing range will exclude the block that was synced,
		// and start syncing from the next block instead
		nonce = 0

		err = m.writeMessages(ctx, basicPayload)
		if err != nil {
			return err
		}
	}

	eg.Go(func() error {
		for {
			select {
			case <-ctx.Done():
				return nil
			case blockNumber, ok := <-blockNumber:
				if !ok {
					return nil
				}
				log.WithFields(log.Fields{
					"block_number": blockNumber,
				}).Info("last synced execution header received in basic channel")
				if blockNumber == 0 {
					continue
				}

				lastVerifiedBlockNumber = lastVerifiedBlockNumber + 1

				log.WithFields(log.Fields{
					"start": lastVerifiedBlockNumber,
					"end":   blockNumber,
				}).Info("fetching basic channel messages")
				basicPayload, err := m.listener.ProcessBasicEvents(ctx, lastVerifiedBlockNumber, blockNumber)
				if err != nil {
					return err
				}

				err = m.writeMessages(ctx, basicPayload)
				if err != nil {
					return err
				}

				lastVerifiedBlockNumber = blockNumber
			}
		}
	})

	return nil
}

func (m *Message) SyncIncentivized(ctx context.Context, eg *errgroup.Group, blockNumber <-chan uint64) error {
	lastVerifiedBlockNumber, err := m.writer.GetLastIncentivizedChannelMessage()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message block number")
	}

	nonce, err := m.writer.GetLastIncentivizedChannelNonce()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message nonce")
	}

	log.WithFields(log.Fields{
		"block_number": lastVerifiedBlockNumber,
		"nonce":        nonce,
	}).Info("last incentivized channel")

	// If the last nonce is set, there could be messages that have not been processed in the same block.
	// Messages that have already been verified will not be reprocessed because they will be filtered out
	// in filterMessagesByLastNonce.
	// Messages after the lastVerifiedBlockNumber will be processed normally in the go routine below.
	if nonce != 0 {
		log.Info("processing incentivized block events for last verified block")
		incentivizedPayload, err := m.listener.ProcessIncentivizedEvents(ctx, lastVerifiedBlockNumber, lastVerifiedBlockNumber)
		if err != nil {
			return err
		}

		incentivizedPayload.Messages = filterMessagesByLastNonce(incentivizedPayload.Messages, nonce)
		// Reset the nonce so that the next block processing range will exclude the block that was synced,
		// and start syncing from the next block instead
		nonce = 0

		log.WithFields(log.Fields{
			"incentivizedPayload": incentivizedPayload,
		}).Info("writing incentivized messages")

		err = m.writeMessages(ctx, incentivizedPayload)
		if err != nil {
			return err
		}
	}

	eg.Go(func() error {
		for {
			select {
			case <-ctx.Done():
				return nil
			case blockNumber, ok := <-blockNumber:
				if !ok {
					return nil
				}
				log.WithFields(log.Fields{
					"block_number": blockNumber,
				}).Info("last synced execution header received in incentivized channel")

				lastVerifiedBlockNumber = lastVerifiedBlockNumber + 1

				log.WithFields(log.Fields{
					"start": lastVerifiedBlockNumber,
					"end":   blockNumber,
				}).Info("fetching incentivized events")

				incentivizedPayload, err := m.listener.ProcessIncentivizedEvents(ctx, lastVerifiedBlockNumber, blockNumber)
				if err != nil {
					return err
				}

				err = m.writeMessages(ctx, incentivizedPayload)
				if err != nil {
					return err
				}

				lastVerifiedBlockNumber = blockNumber
			}
		}
	})

	return nil
}

func (m *Message) writeMessages(ctx context.Context, payload ParachainPayload) error {
	for _, msg := range payload.Messages {
		err := m.writer.WriteToParachain(ctx, msg.Call, msg.Args...)
		if err != nil {
			return err
		}
	}

	return nil
}

func filterMessagesByLastNonce(messages []*chain.EthereumOutboundMessage, nonce uint64) []*chain.EthereumOutboundMessage {
	resultMessages := []*chain.EthereumOutboundMessage{}

	for _, incentivizedMessage := range messages {
		if incentivizedMessage.Nonce <= nonce {
			continue
		}

		resultMessages = append(resultMessages, incentivizedMessage)
	}

	return resultMessages
}
