package message

import (
	"context"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"golang.org/x/sync/errgroup"
)

type Message struct {
	writer    *parachain.ParachainWriter
	listener  *EthereumListener
}

func New(writer *parachain.ParachainWriter, listener *EthereumListener) (*Message, error) {
	return &Message{writer, listener}, nil
}

func (m *Message) Sync(ctx context.Context, eg *errgroup.Group) error {
	lastSyncedBasicBlock, err := m.syncUnprocessedBasicMessages(ctx)
	if err != nil {
		return fmt.Errorf("sync unprocessed basic messages: %w", err)
	}

	ticker := time.NewTicker(time.Second * 20)

	log.WithFields(log.Fields{
		"lastSyncedBasicBlock": lastSyncedBasicBlock,
	}).Info("last synced execution block numbers")

	eg.Go(func() error {
		for {
			executionHeaderState, err := m.writer.GetLastExecutionHeaderState()
			if err != nil {
				return fmt.Errorf("fetch last synced execution header state: %w", err)
			}

			lastSyncedBlockNumber := executionHeaderState.BlockNumber

			if lastSyncedBlockNumber > 0 && lastSyncedBlockNumber > lastSyncedBasicBlock {
				log.WithFields(log.Fields{
					"lastSyncedBasicBlock": lastSyncedBasicBlock,
					"blockNumber":          lastSyncedBlockNumber,
				}).Info("last synced execution block changed, fetching messages")

				if lastSyncedBasicBlock+1 <= lastSyncedBlockNumber {
					err = m.syncBasic(ctx, eg, lastSyncedBasicBlock+1, lastSyncedBlockNumber)
					if err != nil {
						return fmt.Errorf("sync basic messages: %w", err)
					}
				}

				lastSyncedBasicBlock = lastSyncedBlockNumber
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

func (m *Message) sync(ctx context.Context, eg *errgroup.Group, secondLastSyncedBlockNumber, lastSyncedBlockNumber uint64) error {
	log.WithFields(log.Fields{
		"start": secondLastSyncedBlockNumber,
		"end":   lastSyncedBlockNumber,
	}).Info("fetching basic channel messages")
	payload, err := m.listener.ProcessEvents(ctx, secondLastSyncedBlockNumber, lastSyncedBlockNumber)
	if err != nil {
		return err
	}

	return m.writeBasicMessages(ctx, payload)
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

func (m *Message) syncUnprocessedBasicMessages(ctx context.Context) (uint64, error) {
	lastVerifiedBlockNumber, err := m.writer.GetLastBasicChannelBlockNumber()
	if err != nil {
		return 0, fmt.Errorf("fetch last basic channel message block number: %w", err)
	}

	addressNonceMap, err := m.writer.GetLastBasicChannelNonceByAddresses(m.addresses)
	if err != nil {
		return 0, fmt.Errorf("fetch last basic channel message nonce: %w", err)
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
		return lastVerifiedBlockNumber, nil
	}

	log.WithFields(log.Fields{
		"block_number": lastVerifiedBlockNumber,
		"nonces":       addressNonzeroNonceMap,
	}).Info("checking last synced basic channel messages on startup")
	basicPayload, err := m.listener.ProcessEvents(ctx, lastVerifiedBlockNumber, lastVerifiedBlockNumber, addressNonzeroNonceMap)
	if err != nil {
		return 0, err
	}

	basicPayload.Messages = filterMessagesByLastNonces(basicPayload.Messages, addressNonzeroNonceMap)

	err = m.writeBasicMessages(ctx, basicPayload)
	if err != nil {
		return 0, err
	}

	return lastVerifiedBlockNumber, err
}
