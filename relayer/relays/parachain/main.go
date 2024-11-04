package parachain

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"github.com/snowfork/snowbridge/relayer/relays/util"

	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
)

type Relay struct {
	config                *Config
	parachainConn         *parachain.Connection
	relaychainConn        *relaychain.Connection
	ethereumConnWriter    *ethereum.Connection
	ethereumConnBeefy     *ethereum.Connection
	ethereumChannelWriter *EthereumWriter
	beefyListener         *BeefyListener
	parachainWriter       *parachain.ParachainWriter
	beaconHeader          *header.Header
	headerCache           *ethereum.HeaderCache
}

func NewRelay(config *Config, keypair *secp256k1.Keypair) (*Relay, error) {
	log.Info("Creating worker")

	parachainConn := parachain.NewConnection(config.Source.Parachain.Endpoint, nil)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)

	ethereumConnWriter := ethereum.NewConnection(&config.Sink.Ethereum, keypair)
	ethereumConnBeefy := ethereum.NewConnection(&config.Source.Ethereum, keypair)

	// channel for messages from beefy listener to ethereum writer
	var tasks = make(chan *TaskV2, 1)

	ethereumChannelWriter, err := NewEthereumWriter(
		&config.Sink,
		ethereumConnWriter,
		tasks,
		config,
	)
	if err != nil {
		return nil, err
	}

	beefyListener := NewBeefyListener(
		&config.Source,
		&config.Schedule,
		ethereumConnBeefy,
		relaychainConn,
		parachainConn,
		tasks,
	)

	parachainWriter := parachain.NewParachainWriter(
		parachainConn,
		8,
	)
	headerCache, err := ethereum.NewHeaderBlockCache(
		&ethereum.DefaultBlockLoader{Conn: ethereumConnWriter},
	)
	if err != nil {
		return nil, err
	}
	p := protocol.New(config.Source.Beacon.Spec, 20)
	store := store.New(config.Source.Beacon.DataStore.Location, config.Source.Beacon.DataStore.MaxEntries, *p)
	store.Connect()
	beaconAPI := api.NewBeaconClient(config.Source.Beacon.Endpoint, config.Source.Beacon.StateEndpoint)
	beaconHeader := header.New(
		parachainWriter,
		beaconAPI,
		config.Source.Beacon.Spec,
		&store,
		p,
		0, // setting is not used in the execution relay
	)
	return &Relay{
		config:                config,
		parachainConn:         parachainConn,
		relaychainConn:        relaychainConn,
		ethereumConnWriter:    ethereumConnWriter,
		ethereumConnBeefy:     ethereumConnBeefy,
		ethereumChannelWriter: ethereumChannelWriter,
		beefyListener:         beefyListener,
		parachainWriter:       parachainWriter,
		beaconHeader:          &beaconHeader,
		headerCache:           headerCache,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.parachainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return err
	}

	err = relay.ethereumConnWriter.Connect(ctx)
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: writer: %w", err)
	}

	err = relay.ethereumConnBeefy.Connect(ctx)
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: beefy: %w", err)
	}

	err = relay.relaychainConn.Connect(ctx)
	if err != nil {
		return err
	}

	log.Info("Starting beefy listener")
	err = relay.beefyListener.Start(ctx, eg)
	if err != nil {
		return err
	}

	log.Info("Starting ethereum writer")
	err = relay.ethereumChannelWriter.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = relay.parachainWriter.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = relay.startDeliverProof(ctx, eg)
	if err != nil {
		return err
	}

	log.Info("Current relay's ID:", relay.config.Schedule.ID)

	return nil
}

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
						return fmt.Errorf("find event GatewayInboundMessageDispatched0: %w", err)
					}
					err = relay.doSubmit(ctx, event)
					if err != nil {
						return fmt.Errorf("submit delivery proof for GatewayInboundMessageDispatched0: %w", err)
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
) (*contracts.GatewayInboundMessageDispatched0, error) {

	const BlocksPerQuery = 4096

	var event *contracts.GatewayInboundMessageDispatched0

	blockNumber, err := relay.ethereumConnWriter.Client().BlockNumber(ctx)
	if err != nil {
		return event, fmt.Errorf("get last block number: %w", err)
	}

	done := false

	rewardAddress, err := util.HexStringTo32Bytes(relay.config.RewardAddress)
	if err != nil {
		return event, fmt.Errorf("convert to reward address: %w", err)
	}

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

		iter, err := relay.ethereumChannelWriter.gateway.FilterInboundMessageDispatched0(&opts, []uint64{nonce}, [][32]byte{rewardAddress})
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
	event *contracts.GatewayInboundMessageDispatched0,
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

func (relay *Relay) doSubmit(ctx context.Context, ev *contracts.GatewayInboundMessageDispatched0) error {
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
	if err != nil && !errors.Is(err, header.ErrBeaconHeaderNotFinalized) {
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
