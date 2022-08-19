package message

import (
	"context"
	"fmt"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/writer"
	"golang.org/x/sync/errgroup"
)

type Message struct {
	cache    *cache.BeaconCache
	writer   *writer.ParachainWriter
	listener *EthereumListener
}

func New(ctx context.Context, eg *errgroup.Group, cache *cache.BeaconCache, writer *writer.ParachainWriter, configSource *config.SourceConfig, ethconn *ethereum.Connection) (*Message, error) {
	listener := NewEthereumListener(
		configSource,
		ethconn,
	)

	err := listener.Start(ctx, eg)
	if err != nil {
		return &Message{}, err
	}

	return &Message{cache, writer, listener}, nil
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

func (m *Message) SyncMessages(ctx context.Context, blockNumber <-chan uint64) error {
	var err error
	m.cache.LastSyncIncentivizedMessageBlockNumber, err = m.writer.GetLastIncentivizedChannelMessage()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message block number")
	}

	icNonce, err := m.writer.GetLastIncentivizedChannelNonce()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message nonce")
	}

	logrus.WithFields(logrus.Fields{
		"block_number": m.cache.LastSyncIncentivizedMessageBlockNumber,
		"nonce":        icNonce,
	}).Info("last incentivized channel")

	m.cache.LastSyncBasicMessageBlockNumber, err = m.writer.GetLastBasicChannelMessage()
	if err != nil {
		return fmt.Errorf("fetch last basic channel message block number")
	}

	bcNonce, err := m.writer.GetLastIncentivizedChannelNonce()
	if err != nil {
		return fmt.Errorf("fetch last basic channel message nonce")
	}

	logrus.WithFields(logrus.Fields{
		"block_number": m.cache.LastSyncBasicMessageBlockNumber,
		"nonce":        bcNonce,
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
			}).Info("last synced execution header")
			if blockNumber == 0 {
				//if m.cache.LastSyncedHeaderBlockNumber == 0 || m.cache.LastSyncBasicMessageBlockNumber == blockNumber || m.cache.LastSyncBasicMessageBlockNumber == blockNumber {
				continue
			}

			m.cache.LastSyncBasicMessageBlockNumber = m.cache.LastSyncBasicMessageBlockNumber + 1
			m.cache.LastSyncIncentivizedMessageBlockNumber = m.cache.LastSyncIncentivizedMessageBlockNumber + 1

			logrus.WithFields(logrus.Fields{
				"start": m.cache.LastSyncBasicMessageBlockNumber,
				"end":   blockNumber,
			}).Info("fetching basic events")
			basicPayload, err := m.listener.ProcessBasicEvents(ctx, m.cache.LastSyncBasicMessageBlockNumber, blockNumber)
			if err != nil {
				return err
			}

			m.writeMessages(ctx, basicPayload)

			logrus.WithFields(logrus.Fields{
				"start": m.cache.LastSyncIncentivizedMessageBlockNumber,
				"end":   blockNumber,
			}).Info("fetching incentivized events")
			incentivizedPayload, err := m.listener.ProcessIncentivizedEvents(ctx, m.cache.LastSyncIncentivizedMessageBlockNumber, blockNumber)
			if err != nil {
				return err
			}

			m.writeMessages(ctx, incentivizedPayload)

			m.cache.LastSyncBasicMessageBlockNumber = blockNumber
			m.cache.LastSyncIncentivizedMessageBlockNumber = blockNumber
		}
	}

	return nil
}
