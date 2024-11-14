package parachain

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"

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

func NewRelay(config *Config, keypair *secp256k1.Keypair, keypair2 *sr25519.Keypair) (*Relay, error) {
	log.Info("Creating worker")

	parachainConn := parachain.NewConnection(config.Source.Parachain.Endpoint, nil)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)

	ethereumConnWriter := ethereum.NewConnection(&config.Sink.Ethereum, keypair)
	ethereumConnBeefy := ethereum.NewConnection(&config.Source.Ethereum, keypair)

	// channel for messages from beefy listener to ethereum writer
	var tasks = make(chan *Task, 1)

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

	parachainWriterConn := parachain.NewConnection(config.Source.Parachain.Endpoint, keypair2.AsKeyringPair())

	parachainWriter := parachain.NewParachainWriter(
		parachainWriterConn,
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
