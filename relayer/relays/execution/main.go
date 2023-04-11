package execution

import (
	"context"
	"fmt"
	"math/big"
	"sort"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config                *Config
	keypair               *sr25519.Keypair
	paraconn              *parachain.Connection
	ethconn               *ethereum.Connection
	outboundQueueContract *contracts.OutboundQueue
}

func NewRelay(
	config *Config,
	keypair *sr25519.Keypair,
) *Relay {
	return &Relay{
		config:  config,
		keypair: keypair,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	paraconn := parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())
	ethconn := ethereum.NewConnection(r.config.Source.Ethereum.Endpoint, nil)

	err := paraconn.Connect(ctx)
	if err != nil {
		return err
	}
	r.paraconn = paraconn

	err = ethconn.Connect(ctx)
	if err != nil {
		return err
	}
	r.ethconn = ethconn

	writer := parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
	)

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	headerCache, err := ethereum.NewHeaderBlockCache(
		&ethereum.DefaultBlockLoader{Conn: ethconn},
	)
	if err != nil {
		return err
	}

	address := common.HexToAddress(r.config.Source.Contracts.OutboundQueue)
	contract, err := contracts.NewOutboundQueue(address, ethconn.Client())
	if err != nil {
		return err
	}
	r.outboundQueueContract = contract

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-time.After(12 * time.Second):
			log.Info("Polling")

			executionHeaderState, err := writer.GetLastExecutionHeaderState()
			if err != nil {
				return err
			}

			paraNonce, err := r.fetchLatestParachainNonce()
			if err != nil {
				return err
			}

			ethNonce, err := r.fetchEthereumNonce(ctx, executionHeaderState.BlockNumber)
			if err != nil {
				return err
			}

			log.WithFields(log.Fields{
				"paraNonce": paraNonce,
				"ethNonce":  ethNonce,
			}).Info("Polled Nonces")

			if paraNonce == ethNonce {
				continue
			}

			events, err := r.findEvents(ctx, executionHeaderState.BlockNumber, paraNonce+1)

			for _, ev := range events {
				inboundMsg, err := r.makeInboundMessage(ctx, headerCache, ev)
				if err != nil {
					return fmt.Errorf("make outgoing message: %w", err)
				}

				err = writer.WriteToParachainAndWatch(ctx, "EthereumInboundQueue.submit", inboundMsg)
				if err != nil {
					return fmt.Errorf("write to parachain: %w", err)
				}
			}
		}
	}
}

func (r *Relay) fetchLatestParachainNonce() (uint64, error) {
	paraID := r.config.Source.LaneID
	encodedParaID, err := types.EncodeToBytes(r.config.Source.LaneID)
	if err != nil {
		return 0, err
	}

	paraNonceKey, err := types.CreateStorageKey(r.paraconn.Metadata(), "EthereumInboundQueue", "Nonce", encodedParaID, nil)
	if err != nil {
		return 0, fmt.Errorf("create storage key for EthereumInboundQueue.Nonce(%v): %w",
			paraID, err)
	}
	var paraNonce uint64
	ok, err := r.paraconn.API().RPC.State.GetStorageLatest(paraNonceKey, &paraNonce)
	if err != nil {
		return 0, fmt.Errorf("fetch storage EthereumInboundQueue.Nonce(%v): %w",
			paraID, err)
	}
	if !ok {
		paraNonce = 0
	}

	return paraNonce, nil
}

func (r *Relay) fetchEthereumNonce(ctx context.Context, blockNumber uint64) (uint64, error) {
	opts := bind.CallOpts{
		Pending:     false,
		BlockNumber: new(big.Int).SetUint64(blockNumber),
		Context:     ctx,
	}
	nonce, err := r.outboundQueueContract.Nonce(&opts, r.config.Source.LaneID)
	if err != nil {
		return 0, fmt.Errorf("fetch OutboundQueue.Nonce(%v): %w", r.config.Source.LaneID, err)
	}

	return nonce, nil
}

const BlocksPerQuery = 4096

func (r *Relay) findEvents(ctx context.Context, latestFinalizedBlockNumber uint64, start uint64) ([]*contracts.OutboundQueueMessage, error) {

	paraID := r.config.Source.LaneID

	var allEvents []*contracts.OutboundQueueMessage

	blockNumber := latestFinalizedBlockNumber

	for {
		log.Info("loop")

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

		done, events, err := r.findEventsWithFilter(&opts, paraID, start)
		if err != nil {
			return nil, fmt.Errorf("filter events: %w", err)
		}

		if len(events) > 0 {
			allEvents = append(allEvents, events...)
		}

		blockNumber = begin

		if done || begin == 0 {
			break
		}
	}

	sort.SliceStable(allEvents, func(i, j int) bool {
		return allEvents[i].Nonce < allEvents[j].Nonce
	})

	return allEvents, nil
}

func (r *Relay) findEventsWithFilter(opts *bind.FilterOpts, paraID uint32, start uint64) (bool, []*contracts.OutboundQueueMessage, error) {
	iter, err := r.outboundQueueContract.FilterMessage(opts, []uint32{paraID}, []uint64{})
	if err != nil {
		return false, nil, err
	}

	var events []*contracts.OutboundQueueMessage
	done := false

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			if err != nil {
				return false, nil, err
			}
			break
		}

		events = append(events, iter.Event)
		if iter.Event.Nonce == start {
			done = true
			iter.Close()
			break
		}
	}

	return done, events, nil
}

func (r *Relay) makeInboundMessage(
	ctx context.Context,
	headerCache *ethereum.HeaderCache,
	event *contracts.OutboundQueueMessage,
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

	return msg, nil
}
