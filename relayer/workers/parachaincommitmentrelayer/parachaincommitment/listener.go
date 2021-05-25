package parachaincommitment

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/sirupsen/logrus"
	rpcOffchain "github.com/snowfork/go-substrate-rpc-client/v2/rpc/offchain"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Listener struct {
	parachainConnection  *parachain.Connection
	relaychainConnection *relaychain.Connection
	ethereumConnection   *ethereum.Connection
	ethereumConfig       *ethereum.Config
	messages             chan<- interface{}
	log                  *logrus.Entry
}

func NewListener(parachainConnection *parachain.Connection,
	relaychainConnection *relaychain.Connection,
	ethereumConnection *ethereum.Connection,
	ethereumConfig *ethereum.Config,
	messages chan<- interface{},
	log *logrus.Entry,
) *Listener {
	return &Listener{
		parachainConnection:  parachainConnection,
		relaychainConnection: relaychainConnection,
		ethereumConnection:   ethereumConnection,
		ethereumConfig:       ethereumConfig,
		messages:             messages,
		log:                  log,
	}
}

func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {

	blockNumber, err := li.fetchStartBlock()
	if err != nil {
		return nil
	}

	li.catchupMissedCommitments(ctx, blockNumber)

	headers := make(chan types.Header)

	eg.Go(func() error {
		err = li.produceFinalizedHeaders(ctx, blockNumber, headers)
		close(headers)
		return err
	})

	eg.Go(func() error {
		err := li.consumeFinalizedHeaders(ctx, headers)
		close(li.messages)
		return err
	})

	return nil
}

func sleep(ctx context.Context, delay time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(delay):
	}
}

// Fetch the starting block
func (li *Listener) fetchStartBlock() (uint64, error) {
	header, err := li.parachainConnection.GetFinalizedHeader()
	if err != nil {
		li.log.WithError(err).Error("Failed to fetch hash for starting block")
		return 0, err
	}

	return uint64(header.Number), nil
}

var ErrBlockNotReady = errors.New("required result to be 32 bytes, but got 0")

func (li *Listener) produceFinalizedHeaders(ctx context.Context, startBlock uint64, headers chan<- types.Header) error {
	current := startBlock
	retryInterval := time.Duration(6) * time.Second
	for {
		select {
		case <-ctx.Done():
			li.log.Info("Shutting down producer of finalized headers")
			return ctx.Err()
		default:
			finalizedHeader, err := li.parachainConnection.GetFinalizedHeader()
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch header for finalized head")
				return err
			}

			if current > uint64(finalizedHeader.Number) {
				li.log.WithFields(logrus.Fields{
					"block":  current,
					"latest": finalizedHeader.Number,
				}).Trace("Block is not yet finalized")
				sleep(ctx, retryInterval)
				continue
			}

			hash, err := li.parachainConnection.GetAPI().RPC.Chain.GetBlockHash(current)
			if err != nil {
				if err.Error() == ErrBlockNotReady.Error() {
					sleep(ctx, retryInterval)
					continue
				} else {
					li.log.WithError(err).Error("Failed to fetch block hash")
					return err
				}
			}

			header, err := li.parachainConnection.GetAPI().RPC.Chain.GetHeader(hash)
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch header")
				return err
			}

			headers <- *header
			current = current + 1
		}
	}
}

func (li *Listener) consumeFinalizedHeaders(ctx context.Context, headers <-chan types.Header) error {
	if li.messages == nil {
		li.log.Info("Not polling events since channel is nil")
		return nil
	}

	for {
		select {
		case <-ctx.Done():
			li.log.Info("Shutting down consumer of finalized headers")
			return ctx.Err()
		case header, ok := <-headers:
			// check if headers channel has closed
			if !ok {
				return nil
			}
			err := li.processHeader(ctx, header)
			if err != nil {
				return err
			}
		}
	}
}

func (li *Listener) processHeader(ctx context.Context, header types.Header) error {

	li.log.WithFields(logrus.Fields{
		"blockNumber": header.Number,
	}).Debug("Processing block")

	digestItem, err := getAuxiliaryDigestItem(header.Digest)
	if err != nil {
		return err
	}

	if digestItem == nil || !digestItem.IsCommitment {
		return nil
	}

	li.log.WithFields(logrus.Fields{
		"block":      header.Number,
		"channelID":  digestItem.AsCommitment.ChannelID,
		"commitment": digestItem.AsCommitment.Hash.Hex(),
	}).Debug("Found commitment hash in header digest")

	err = li.processCommitment(digestItem.AsCommitment.ChannelID, digestItem.AsCommitment.Hash)
	if err != nil {
		return err
	}
	return nil
}

func (li *Listener) processCommitment(channel chainTypes.ChannelID, commitment types.H256) error {
	data, err := li.getOffchainDataForCommitment(channel, commitment)
	if err != nil {
		li.log.WithError(err).Error("Failed to read commitment from offchain storage")
		return err
	}

	if data != nil {
		li.log.WithFields(logrus.Fields{
			"commitmentSizeBytes": len(*data),
		}).Debug("Retrieved commitment from offchain storage")
	} else {
		li.log.WithError(err).Error("Commitment not found in offchain storage")
		return err
	}

	switch {
	case channel.IsBasic:
		var messages []chainTypes.BasicOutboundChannelMessage

		err = types.DecodeFromBytes(*data, &messages)
		if err != nil {
			li.log.WithError(err).Error("Failed to decode commitment messages")
			return err
		}

		message := chain.SubstrateOutboundBasicMessage{
			Commitment: commitment,
			Messages:   messages,
		}

		li.messages <- message
	case channel.IsIncentivized:
		var messages []chainTypes.IncentivizedOutboundChannelMessage

		err = types.DecodeFromBytes(*data, &messages)
		if err != nil {
			li.log.WithError(err).Error("Failed to decode commitment messages")
			return err
		}

		message := chain.SubstrateOutboundIncentivizedMessage{
			Commitment: commitment,
			Messages:   messages,
		}

		li.messages <- message

	default:
		return fmt.Errorf("Unknown channel")

	}

	return nil
}

func getAuxiliaryDigestItem(digest types.Digest) (*chainTypes.AuxiliaryDigestItem, error) {
	for _, digestItem := range digest {
		if digestItem.IsOther {
			var auxDigestItem chainTypes.AuxiliaryDigestItem
			err := types.DecodeFromBytes(digestItem.AsOther, &auxDigestItem)
			if err != nil {
				return nil, err
			}
			return &auxDigestItem, nil
		}
	}
	return nil, nil
}

func (li *Listener) getOffchainDataForCommitment(channel chainTypes.ChannelID, commitment types.H256) (*types.StorageDataRaw, error) {
	storageKey, err := parachain.MakeStorageKey(channel, commitment)
	if err != nil {
		return nil, err
	}

	data, err := li.parachainConnection.GetAPI().RPC.Offchain.LocalStorageGet(rpcOffchain.Persistent, storageKey)
	if err != nil {
		return nil, err
	}

	return data, nil
}
