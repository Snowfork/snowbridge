package message

import (
	"context"
	"fmt"

	"github.com/sirupsen/logrus"
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

func (m *Message) SyncBasic(ctx context.Context, blockNumber <-chan uint64) error {
	lastVerifiedBlockNumber, err := m.writer.GetLastBasicChannelMessage()
	if err != nil {
		return fmt.Errorf("fetch last basic channel message block number")
	}

	nonce, err := m.writer.GetLastBasicChannelNonce()
	if err != nil {
		return fmt.Errorf("fetch last basic channel message nonce")
	}

	logrus.WithFields(logrus.Fields{
		"block_number": lastVerifiedBlockNumber,
		"nonce":        nonce,
	}).Info("last basic channel")

	for {
		select {
		case <-ctx.Done():
			return nil
		case blockNumber, ok := <-blockNumber:
			if !ok {
				return nil
			}
			logrus.WithFields(logrus.Fields{
				"block_number": blockNumber,
			}).Info("last synced execution header received in basic channel")
			if blockNumber == 0 {
				continue
			}

			lastVerifiedBlockNumber = lastVerifiedBlockNumber + 1

			logrus.WithFields(logrus.Fields{
				"start": lastVerifiedBlockNumber,
				"end":   blockNumber,
			}).Info("fetching basic channel messages")
			basicPayload, err := m.listener.ProcessBasicEvents(ctx, lastVerifiedBlockNumber, blockNumber)
			if err != nil {
				return err
			}

			m.writeMessages(ctx, basicPayload)

			lastVerifiedBlockNumber = blockNumber
		}
	}

	return nil
}

func (m *Message) SyncIncentivized(ctx context.Context, blockNumber <-chan uint64) error {
	lastVerifiedBlockNumber, err := m.writer.GetLastIncentivizedChannelMessage()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message block number")
	}

	nonce, err := m.writer.GetLastIncentivizedChannelNonce()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message nonce")
	}

	logrus.WithFields(logrus.Fields{
		"block_number": lastVerifiedBlockNumber,
		"nonce":        nonce,
	}).Info("last incentivized channel")

	for {
		select {
		case <-ctx.Done():
			return nil
		case blockNumber, ok := <-blockNumber:
			if !ok {
				return nil
			}
			logrus.WithFields(logrus.Fields{
				"block_number": blockNumber,
			}).Info("last synced execution header received in incentivized channel")

			lastVerifiedBlockNumber = lastVerifiedBlockNumber + 1

			logrus.WithFields(logrus.Fields{
				"start": lastVerifiedBlockNumber,
				"end":   blockNumber,
			}).Info("fetching incentivized events")

			incentivizedPayload, err := m.listener.ProcessIncentivizedEvents(ctx, lastVerifiedBlockNumber, blockNumber)
			if err != nil {
				return err
			}

			m.writeMessages(ctx, incentivizedPayload)

			lastVerifiedBlockNumber = blockNumber
		}
	}

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
