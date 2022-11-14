package message

import (
	"context"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/execution/config"
	"golang.org/x/sync/errgroup"
)

type Message struct {
	writer    *parachain.ParachainWriter
	listener  *EthereumListener
	addresses []common.Address
}

func New(ctx context.Context, eg *errgroup.Group, writer *parachain.ParachainWriter, configSource *config.SourceConfig, ethconn *ethereum.Connection) (*Message, error) {
	addresses, err := configSource.GetBasicChannelAddresses()
	if err != nil {
		return nil, err
	}

	listener := NewEthereumListener(
		configSource,
		ethconn,
	)

	err = listener.Start(ctx, eg)
	if err != nil {
		return &Message{}, err
	}

	return &Message{writer, listener, addresses}, nil
}

func (m *Message) Sync(ctx context.Context, eg *errgroup.Group) error {
	executionHeaderState, err := m.writer.GetLastExecutionHeaderState()
	if err != nil {
		return fmt.Errorf("fetch last synced execution header state: %w", err)
	}

	lastSyncedBlockNumber := executionHeaderState.BlockNumber

	log.WithFields(log.Fields{
		"blockNumber": lastSyncedBlockNumber,
	}).Info("last synced execution block number")

	m.syncUnprocessedIncentivizedMessages(ctx)

	m.syncUnprocessedBasicMessages(ctx)

	ticker := time.NewTicker(time.Second * 20)

	secondLastSyncedBlockNumber := lastSyncedBlockNumber

	eg.Go(func() error {
		for {
			executionHeaderState, err := m.writer.GetLastExecutionHeaderState()
			if err != nil {
				return fmt.Errorf("fetch last synced execution header state: %w", err)
			}

			lastSyncedBlockNumber := executionHeaderState.BlockNumber

			if lastSyncedBlockNumber > 0 && lastSyncedBlockNumber != secondLastSyncedBlockNumber {
				log.WithFields(log.Fields{
					"lastBlockNumber": lastSyncedBlockNumber,
				}).Info("last synced execution block changed, fetching messages")

				err = m.syncIncentivized(ctx, eg, secondLastSyncedBlockNumber+1, lastSyncedBlockNumber)
				if err != nil {
					return fmt.Errorf("sync incentivized messages: %w", err)
				}

				err = m.syncBasic(ctx, eg, secondLastSyncedBlockNumber+1, lastSyncedBlockNumber)
				if err != nil {
					return fmt.Errorf("sync basic messages: %w", err)
				}

				secondLastSyncedBlockNumber = lastSyncedBlockNumber
			} else {
				log.WithFields(log.Fields{
					"lastBlockNumber": lastSyncedBlockNumber,
				}).Info("last synced execution block unchanged")
			}

			select {
			case <-ctx.Done():
				return nil
			case <-ticker.C:
				continue
			}
		}
	})

	return nil
}

func (m *Message) syncBasic(ctx context.Context, eg *errgroup.Group, secondLastSyncedBlockNumber, lastSyncedBlockNumber uint64) error {
	addressNonceMap := make(map[common.Address]uint64, len(m.addresses))
	for _, address := range m.addresses {
		addressNonceMap[address] = 1
	}

	log.WithFields(log.Fields{
		"start": secondLastSyncedBlockNumber,
		"end":   lastSyncedBlockNumber,
	}).Info("fetching basic channel messages")
	basicPayload, err := m.listener.ProcessBasicEvents(ctx, secondLastSyncedBlockNumber, lastSyncedBlockNumber, addressNonceMap)
	if err != nil {
		return err
	}

	return m.writeBasicMessages(ctx, basicPayload)
}

func (m *Message) syncIncentivized(ctx context.Context, eg *errgroup.Group, secondLastSyncedBlockNumber, lastSyncedBlockNumber uint64) error {
	log.WithFields(log.Fields{
		"start": secondLastSyncedBlockNumber,
		"end":   lastSyncedBlockNumber,
	}).Info("fetching incentivized channel messages")

	incentivizedPayload, err := m.listener.ProcessIncentivizedEvents(ctx, secondLastSyncedBlockNumber, lastSyncedBlockNumber)
	if err != nil {
		return err
	}

	return m.writeIncentivizedMessages(ctx, incentivizedPayload)
}

func (m *Message) writeBasicMessages(ctx context.Context, payload ParachainPayload) error {
	log.WithField("count", len(payload.Messages)).Info("writing basic messages")

	for _, msg := range payload.Messages {
		err := m.writer.WriteToParachainAndWatch(ctx, msg.Call, msg.Args...)
		if err != nil {
			return err
		}

		err = m.checkMessageVerificationResult(msg.Origin, msg.Nonce)
		if err != nil {
			return err
		}
	}

	return nil
}

func (m *Message) checkMessageVerificationResult(msgAddress common.Address, msgNonce uint64) error {
	nonce, err := m.writer.GetLastBasicChannelNonceByAddress(msgAddress)
	if err != nil {
		return fmt.Errorf("fetch last basic channel message nonces by addresses: %w", err)
	}

	if nonce != msgNonce {
		return fmt.Errorf("last basic message verification failed for address %s (expected nonce: %d, actual nonce: %d)", msgAddress, msgNonce, nonce)
	}

	log.WithFields(log.Fields{"nonce": msgNonce, "address": msgAddress}).Info("basic message verified successfully")
	return nil
}

func (m *Message) writeIncentivizedMessages(ctx context.Context, payload ParachainPayload) error {
	log.WithField("count", len(payload.Messages)).Info("writing incentivized messages")

	for _, msg := range payload.Messages {
		err := m.writer.WriteToParachainAndWatch(ctx, msg.Call, msg.Args...)
		if err != nil {
			return err
		}

		lastNonce, err := m.writer.GetLastIncentivizedChannelNonce()
		if err != nil {
			return err
		}

		if lastNonce != msg.Nonce {
			return fmt.Errorf("last incentivized message verification failed (nonce: %d)", lastNonce)
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

func filterMessagesByLastNonces(messages []*chain.EthereumOutboundMessage, addressNonceMap map[common.Address]uint64) []*chain.EthereumOutboundMessage {
	resultMessages := []*chain.EthereumOutboundMessage{}

	for _, basicMessage := range messages {
		if basicMessage.Nonce <= addressNonceMap[basicMessage.Origin] {
			continue
		}

		resultMessages = append(resultMessages, basicMessage)
	}

	return resultMessages
}

func (m *Message) syncUnprocessedBasicMessages(ctx context.Context) error {
	lastVerifiedBlockNumber, err := m.writer.GetLastBasicChannelBlockNumber()
	if err != nil {
		return fmt.Errorf("fetch last basic channel message block number: %w", err)
	}

	addressNonceMap, err := m.writer.GetLastBasicChannelNonceByAddresses(m.addresses)
	if err != nil {
		return fmt.Errorf("fetch last basic channel message nonce: %w", err)
	}

	addressNonzeroNonceMap := make(map[common.Address]uint64, len(addressNonceMap))
	for address, nonce := range addressNonceMap {
		log.WithFields(log.Fields{
			"block_number": lastVerifiedBlockNumber,
			"address":      address,
			"nonce":        nonce,
		}).Info("last basic channel")

		if nonce != 0 {
			addressNonzeroNonceMap[address] = nonce
		}
	}

	if len(addressNonzeroNonceMap) == 0 {
		return nil
	}

	log.Info("processing basic block events for last verified block")
	basicPayload, err := m.listener.ProcessBasicEvents(ctx, lastVerifiedBlockNumber, lastVerifiedBlockNumber, addressNonzeroNonceMap)
	if err != nil {
		return err
	}

	basicPayload.Messages = filterMessagesByLastNonces(basicPayload.Messages, addressNonzeroNonceMap)

	return m.writeBasicMessages(ctx, basicPayload)
}

func (m *Message) syncUnprocessedIncentivizedMessages(ctx context.Context) error {
	lastVerifiedBlockNumber, err := m.writer.GetLastIncentivizedChannelBlockNumber()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message block number: %w", err)
	}

	nonce, err := m.writer.GetLastIncentivizedChannelNonce()
	if err != nil {
		return fmt.Errorf("fetch last incentivized channel message nonce: %w", err)
	}

	log.WithFields(log.Fields{
		"block_number": lastVerifiedBlockNumber,
		"nonce":        nonce,
	}).Info("last incentivized channel")

	// If the last nonce is set, there could be messages that have not been processed in the same block.
	// Messages that have already been verified will not be reprocessed because they will be filtered out
	// in filterMessagesByLastNonce.
	// Messages after the lastVerifiedBlockNumber will be processed normally in the go routine below.
	if nonce == 0 {
		return nil
	}

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

	return m.writeIncentivizedMessages(ctx, incentivizedPayload)
}
