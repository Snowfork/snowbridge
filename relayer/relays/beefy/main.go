package beefy

import (
	"context"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/store"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	config                  *Config
	relaychainConn          *relaychain.Connection
	ethereumConn            *ethereum.Connection
	beefyEthereumListener   *BeefyEthereumListener
	beefyRelaychainListener *BeefyRelaychainListener
	beefyEthereumWriter     *BeefyEthereumWriter
	beefyDB                 *store.Database
	beefyMessages           chan store.BeefyRelayInfo
	ethHeaders              chan chain.Header
}

func NewRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*Relay, error) {
	log.Info("Relay created")

	dbMessages := make(chan store.DatabaseCmd)
	beefyDB := store.NewDatabase(dbMessages)

	err := beefyDB.Initialize()
	if err != nil {
		return nil, err
	}

	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	ethereumConn := ethereum.NewConnection(config.Sink.Ethereum.Endpoint, ethereumKeypair)

	beefyMessages := make(chan store.BeefyRelayInfo)
	ethHeaders := make(chan chain.Header)

	beefyEthereumListener := NewBeefyEthereumListener(&config.Sink,
		ethereumConn, beefyDB, beefyMessages, dbMessages, ethHeaders)

	beefyEthereumWriter := NewBeefyEthereumWriter(&config.Sink, ethereumConn,
		beefyDB, dbMessages, beefyMessages)

	beefyRelaychainListener := NewBeefyRelaychainListener(
		config,
		relaychainConn,
		beefyMessages,
	)

	return &Relay{
		config:                  config,
		relaychainConn:          relaychainConn,
		beefyEthereumListener:   beefyEthereumListener,
		ethereumConn:            ethereumConn,
		beefyEthereumWriter:     beefyEthereumWriter,
		beefyRelaychainListener: beefyRelaychainListener,
		beefyDB:                 beefyDB,
		beefyMessages:           beefyMessages,
		ethHeaders:              ethHeaders,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.beefyDB.Start(ctx, eg)
	if err != nil {
		log.WithError(err).Error("Failed to start database")
		return err
	}

	err = relay.relaychainConn.Connect(ctx)
	if err != nil {
		return err
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return err
	}

	latestBeefyBlock, err := relay.beefyEthereumListener.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = relay.beefyRelaychainListener.Start(ctx, eg, latestBeefyBlock)
	if err != nil {
		return err
	}

	err = relay.beefyEthereumWriter.Start(ctx, eg)
	if err != nil {
		return err
	}

	return nil
}
