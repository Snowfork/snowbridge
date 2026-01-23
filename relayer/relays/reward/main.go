package reward

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"math/big"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	beaconstate "github.com/snowfork/snowbridge/relayer/relays/beacon-state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config          *Config
	keypair         *sr25519.Keypair
	paraconn        *parachain.Connection
	ethconn         *ethereum.Connection
	gatewayContract *contracts.Gateway
	beaconHeader    *header.Header
	headerCache     *ethereum.HeaderCache
	writer          *parachain.ParachainWriter
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
	ethconn := ethereum.NewConnection(&r.config.Source.Ethereum, nil)

	err := paraconn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(r.config.Sink.Parachain.HeartbeatSecs))
	if err != nil {
		return err
	}
	r.paraconn = paraconn

	err = ethconn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(r.config.Source.Ethereum.HeartbeatSecs))
	if err != nil {
		return err
	}
	r.ethconn = ethconn

	r.writer = parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
	)

	err = r.writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	headerCache, err := ethereum.NewHeaderBlockCache(
		&ethereum.DefaultBlockLoader{Conn: ethconn},
	)
	if err != nil {
		return err
	}
	r.headerCache = headerCache

	address := common.HexToAddress(r.config.Source.Contracts.Gateway)
	contract, err := contracts.NewGateway(address, ethconn.Client())
	if err != nil {
		return err
	}
	r.gatewayContract = contract

	p := protocol.New(r.config.Source.Beacon.Spec, r.config.Sink.Parachain.HeaderRedundancy)

	beaconAPI := api.NewBeaconClient(r.config.Source.Beacon.Endpoint)

	var stateServiceClient syncer.StateServiceClient
	if r.config.Source.Beacon.StateServiceEndpoint != "" {
		stateServiceClient = beaconstate.NewClient(r.config.Source.Beacon.StateServiceEndpoint)
		log.WithField("endpoint", r.config.Source.Beacon.StateServiceEndpoint).Info("Using beacon state service for proof generation")
	}

	beaconHeader := header.New(
		r.writer,
		beaconAPI,
		r.config.Source.Beacon.Spec,
		p,
		0, // setting is not used in the reward relay
		stateServiceClient,
	)
	r.beaconHeader = &beaconHeader

	if err != nil {
		return err
	}

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-time.After(60 * time.Second):
			orders, err := r.findOrderUndelivered(ctx)
			if err != nil {
				return fmt.Errorf("find undelivered order: %w", err)
			}
			rewardAddress, err := util.HexStringTo32Bytes(r.config.RewardAddress)
			if err != nil {
				return fmt.Errorf("convert to reward address: %w", err)
			}
			for _, order := range orders {
				event, err := r.findEvent(ctx, order.Nonce)
				if err != nil {
					return fmt.Errorf("find event GatewayInboundMessageDispatched: %w", err)
				}
				if event.RewardAddress != rewardAddress {
					log.Info("order is not from the current relayer, just ignore")
					continue
				}
				err = r.doSubmit(ctx, event)
				if err != nil {
					return fmt.Errorf("submit delivery proof for GatewayInboundMessageDispatched: %w", err)
				}
			}
		}
	}
}

func (r *Relay) isNonceRelayed(ctx context.Context, nonce uint64) (bool, error) {
	options := bind.CallOpts{
		Pending: true,
		Context: ctx,
	}
	isRelayed, err := r.gatewayContract.V2IsDispatched(&options, nonce)
	if err != nil {
		return isRelayed, fmt.Errorf("check nonce from gateway contract: %w", err)
	}
	return isRelayed, nil
}

func (r *Relay) findOrderUndelivered(
	ctx context.Context,
) ([]*parachain.PendingOrder, error) {
	storageKey := types.NewStorageKey(types.CreateStorageKeyPrefix("EthereumOutboundQueueV2", "PendingOrders"))
	keys, err := r.paraconn.API().RPC.State.GetKeysLatest(storageKey)
	if err != nil {
		return nil, fmt.Errorf("fetch nonces from PendingOrders start with key '%v': %w", storageKey, err)
	}
	var undeliveredOrders []*parachain.PendingOrder
	for _, key := range keys {
		var undeliveredOrder parachain.PendingOrder
		value, err := r.paraconn.API().RPC.State.GetStorageRawLatest(key)
		if err != nil {
			return nil, fmt.Errorf("fetch value of pendingOrder with key '%v': %w", key, err)
		}
		decoder := scale.NewDecoder(bytes.NewReader(*value))
		err = decoder.Decode(&undeliveredOrder)
		if err != nil {
			return nil, fmt.Errorf("decode order error: %w", err)
		}
		isRelayed, err := r.isNonceRelayed(ctx, uint64(undeliveredOrder.Nonce))
		if err != nil {
			return nil, fmt.Errorf("check nonce relayed: %w", err)
		}
		if isRelayed {
			log.WithFields(log.Fields{
				"nonce": uint64(undeliveredOrder.Nonce),
			}).Debug("Relayed but not delivered to BH")
			undeliveredOrders = append(undeliveredOrders, &undeliveredOrder)
		}
	}
	return undeliveredOrders, nil
}

func (relay *Relay) findEvent(
	ctx context.Context,
	nonce uint64,
) (*contracts.GatewayInboundMessageDispatched, error) {

	const BlocksPerQuery = 4096

	var event *contracts.GatewayInboundMessageDispatched

	blockNumber, err := relay.ethconn.Client().BlockNumber(ctx)
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

		iter, err := relay.gatewayContract.FilterInboundMessageDispatched(&opts, []uint64{nonce})
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

	blockHeader, err := relay.ethconn.Client().HeaderByNumber(ctx, nextBlockNumber)
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

	inboundMsg.Proof.ExecutionProof = proof.HeaderPayload

	log.WithFields(logrus.Fields{
		"EventLog": inboundMsg.EventLog,
		"Proof":    inboundMsg.Proof,
	}).Debug("Generated message from Ethereum log")

	err = relay.writer.WriteToParachainAndWatch(ctx, "EthereumOutboundQueueV2.submit_delivery_receipt", inboundMsg)
	if err != nil {
		return fmt.Errorf("submit message to outbound queue v2: %w", err)
	}

	logger.Info("v2 inbound message executed successfully")

	return nil
}
