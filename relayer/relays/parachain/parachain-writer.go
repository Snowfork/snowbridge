package parachain

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"golang.org/x/sync/errgroup"
)

func (relay *Relay) startDeliverProof(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		for {
			select {
			case <-ctx.Done():
				return nil
			case <-time.After(60 * time.Second):
				orders, err := relay.beefyListener.scanner.findOrderUndelivered(ctx)
				if err != nil {
					return fmt.Errorf("find undelivered order: %w", err)
				}
				for _, order := range orders {
					event, err := relay.findEvent(ctx, order.Nonce)
					if err != nil {
						return fmt.Errorf("find event GatewayInboundMessageDispatched: %w", err)
					}
					err = relay.doSubmit(ctx, event)
					if err != nil {
						return fmt.Errorf("submit delivery proof for GatewayInboundMessageDispatched: %w", err)
					}
				}
			}
		}
	})
	return nil
}

// Todo: Improve scan algorithm
func (relay *Relay) findEvent(
	ctx context.Context,
	nonce uint64,
) (*contracts.GatewayInboundMessageDispatched, error) {

	const BlocksPerQuery = 4096

	var event *contracts.GatewayInboundMessageDispatched

	blockNumber, err := relay.ethereumConnWriter.Client().BlockNumber(ctx)
	if err != nil {
		return event, fmt.Errorf("get last block number: %w", err)
	}

	done := false

	for {
		var begin uint64
		if blockNumber < BlocksPerQuery {
			begin = 0
		} else {
			begin = blockNumber - BlocksPerQuery
		}

		opts := bind.FilterOpts{
			Start:   begin,
			End:     &blockNumber,
			Context: ctx,
		}

		iter, err := relay.ethereumChannelWriter.gateway.FilterInboundMessageDispatched(&opts, []uint64{nonce})
		if err != nil {
			return event, fmt.Errorf("iter dispatch event: %w", err)
		}

		for {
			more := iter.Next()
			if !more {
				err = iter.Error()
				if err != nil {
					return event, fmt.Errorf("iter dispatch event: %w", err)
				}
				break
			}
			if iter.Event.Nonce == nonce {
				event = iter.Event
				done = true
				break
			}
		}

		if done {
			iter.Close()
		}

		blockNumber = begin

		if done || begin == 0 {
			break
		}
	}

	return event, nil
}

func (relay *Relay) makeInboundMessage(
	ctx context.Context,
	headerCache *ethereum.HeaderCache,
	event *contracts.GatewayInboundMessageDispatched,
) (*parachain.Message, error) {
	receiptTrie, err := headerCache.GetReceiptTrie(ctx, event.Raw.BlockHash)
	if err != nil {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to get receipt trie for event")
		return nil, err
	}

	msg, err := ethereum.MakeMessageFromEvent(&event.Raw, receiptTrie)
	if err != nil {
		log.WithFields(logrus.Fields{
			"address":     event.Raw.Address.Hex(),
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).WithError(err).Error("Failed to generate message from ethereum event")
		return nil, err
	}

	log.WithFields(logrus.Fields{
		"blockHash":   event.Raw.BlockHash.Hex(),
		"blockNumber": event.Raw.BlockNumber,
		"txHash":      event.Raw.TxHash.Hex(),
	}).Info("found message")

	return msg, nil
}

func (relay *Relay) doSubmit(ctx context.Context, ev *contracts.GatewayInboundMessageDispatched) error {
	inboundMsg, err := relay.makeInboundMessage(ctx, relay.headerCache, ev)
	if err != nil {
		return fmt.Errorf("make outgoing message: %w", err)
	}

	logger := log.WithFields(log.Fields{
		"ethNonce":    ev.Nonce,
		"msgNonce":    ev.Nonce,
		"address":     ev.Raw.Address.Hex(),
		"blockHash":   ev.Raw.BlockHash.Hex(),
		"blockNumber": ev.Raw.BlockNumber,
		"txHash":      ev.Raw.TxHash.Hex(),
		"txIndex":     ev.Raw.TxIndex,
	})

	nextBlockNumber := new(big.Int).SetUint64(ev.Raw.BlockNumber + 1)

	blockHeader, err := relay.ethereumConnWriter.Client().HeaderByNumber(ctx, nextBlockNumber)
	if err != nil {
		return fmt.Errorf("get block header: %w", err)
	}

	proof, err := relay.beaconHeader.FetchExecutionProof(*blockHeader.ParentBeaconRoot, false)
	if errors.Is(err, header.ErrBeaconHeaderNotFinalized) || proof.HeaderPayload.ExecutionBranch == nil {
		logger.Info("event block is not finalized yet")
		return nil
	}
	if err != nil {
		return fmt.Errorf("fetch execution header proof: %w", err)
	}

	err = relay.writeToParachain(ctx, proof, inboundMsg)
	if err != nil {
		return fmt.Errorf("write to parachain: %w", err)
	}

	logger.Info("inbound message executed successfully")

	return nil
}

func (relay *Relay) writeToParachain(ctx context.Context, proof scale.ProofPayload, inboundMsg *parachain.Message) error {
	inboundMsg.Proof.ExecutionProof = proof.HeaderPayload

	log.WithFields(logrus.Fields{
		"EventLog": inboundMsg.EventLog,
		"Proof":    inboundMsg.Proof,
	}).Debug("Generated message from Ethereum log")

	err := relay.parachainWriter.WriteToParachainAndWatch(ctx, "EthereumOutboundQueueV2.submit_delivery_proof", inboundMsg)
	if err != nil {
		return fmt.Errorf("submit message to outbound queue v2: %w", err)
	}

	return nil
}
