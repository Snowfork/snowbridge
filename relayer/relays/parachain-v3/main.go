package parachainV3

import (
	"context"
	"fmt"
	"os"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/config"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"

	"github.com/snowfork/snowbridge/relayer/ofac"

	log "github.com/sirupsen/logrus"
)

// defaultDryRunRPCURL is the forked-mainnet endpoint used for aggregate3 simulation when a
// dry-run RPC is enabled but no explicit DRY_RUN_RPC_URL override is set.
const defaultDryRunRPCURL = "https://fork-mainnet.snowbridge.network"

// newDryRunConnection builds a dedicated simulation connection for aggregate3 dry-runs.
// It is enabled only when FORKED_PROVIDER_API_KEY is set (the X-API-Key the dry-run RPC
// requires); otherwise it returns nil and simulation falls back to the broadcast conn.
// The endpoint defaults to the forked-mainnet RPC and can be overridden with DRY_RUN_RPC_URL.
func newDryRunConnection(sink *config.EthereumConfig, keypair *secp256k1.Keypair) *ethereum.Connection {
	apiKey := os.Getenv("FORKED_PROVIDER_API_KEY")
	if apiKey == "" {
		return nil
	}
	endpoint := os.Getenv("DRY_RUN_RPC_URL")
	if endpoint == "" {
		endpoint = defaultDryRunRPCURL
	}
	dryRunCfg := &config.EthereumConfig{
		Endpoint:      endpoint,
		HeartbeatSecs: sink.HeartbeatSecs,
		Headers:       map[string]string{"X-API-Key": apiKey},
	}
	log.WithField("endpoint", endpoint).Info("Dry-run RPC enabled for aggregate3 simulation")
	return ethereum.NewConnection(dryRunCfg, keypair)
}

type InstantRelay struct {
	config                *Config
	parachainConn         *parachain.Connection
	relaychainConn        *relaychain.Connection
	ethereumConnWriter    *ethereum.Connection
	ethereumConnBeefy     *ethereum.Connection
	ethereumConnDryRun    *ethereum.Connection
	ethereumChannelWriter *EthereumWriter
	beefyListener         *BeefyListener
	beefyInstantSyncer    *BeefyInstantSyncer
}

func NewInstantRelay(config *Config, beefyConfig *beefy.Config, keypair *secp256k1.Keypair) (*InstantRelay, error) {
	if err := config.ValidateForInstantRelay(); err != nil {
		return nil, err
	}

	log.Info("Creating worker")

	parachainConn := parachain.NewConnection(config.Source.Parachain.ParseEndpoints(), nil)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.ParseEndpoints(), nil)

	ethereumConnWriter := ethereum.NewConnection(&config.Sink.Ethereum, keypair)
	ethereumConnBeefy := ethereum.NewConnection(&config.Source.Ethereum, keypair)
	ethereumConnDryRun := newDryRunConnection(&config.Sink.Ethereum, keypair)

	ofacClient := ofac.New(config.OFAC.Enabled, config.OFAC.ApiKey)

	deadNonces, err := error_tracking.NewDeadNonceStore(config.DeadNoncesPath)
	if err != nil {
		return nil, fmt.Errorf("init dead-nonce store: %w", err)
	}

	// channel for messages from beefy listener to ethereum writer
	var tasks = make(chan *Task, 1)

	ethereumChannelWriter, err := NewEthereumWriter(
		&config.Sink,
		ethereumConnWriter,
		ethereumConnBeefy,
		ethereumConnDryRun,
		tasks,
		config,
	)
	if err != nil {
		return nil, err
	}

	beefyListener := NewBeefyListener(
		&config.Source,
		ethereumConnBeefy,
		relaychainConn,
		parachainConn,
		ofacClient,
		tasks,
		config.IgnoreNonces,
		deadNonces,
	)

	beefyOnDemandRelay, err := beefy.NewOnDemandRelay(beefyConfig, keypair)
	if err != nil {
		return nil, err
	}

	beefyInstantSyncer, err := NewBeefyInstantSyncer(
		config,
		beefyListener,
		beefyOnDemandRelay,
		ethereumChannelWriter,
	)
	if err != nil {
		return nil, err
	}

	return &InstantRelay{
		config:                config,
		parachainConn:         parachainConn,
		relaychainConn:        relaychainConn,
		ethereumConnWriter:    ethereumConnWriter,
		ethereumConnBeefy:     ethereumConnBeefy,
		ethereumConnDryRun:    ethereumConnDryRun,
		ethereumChannelWriter: ethereumChannelWriter,
		beefyListener:         beefyListener,
		beefyInstantSyncer:    beefyInstantSyncer,
	}, nil
}

func (relay *InstantRelay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.parachainConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.Parachain.HeartbeatSecs))
	if err != nil {
		return err
	}

	err = relay.ethereumConnWriter.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Sink.Ethereum.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: writer: %w", err)
	}

	err = relay.ethereumConnBeefy.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.Ethereum.HeartbeatSecs))
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: beefy: %w", err)
	}

	if relay.ethereumConnDryRun != nil {
		// heartBeat=0: the dry-run RPC is a stateless HTTPS endpoint, so connect once with
		// no heartbeat/reconnect goroutine (each eth_call dials per-request anyway).
		err = relay.ethereumConnDryRun.ConnectWithHeartBeat(ctx, eg, 0)
		if err != nil {
			// Don't take down the relayer: fall back to simulating on the broadcast conn.
			log.WithError(err).Warn("unable to connect to dry-run RPC; aggregate3 simulation will use the broadcast RPC")
			relay.ethereumChannelWriter.dryRunConn = nil
		}
	}

	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(relay.config.Source.Polkadot.HeartbeatSecs))
	if err != nil {
		return err
	}

	log.Info("Starting beefy instant listener")
	err = relay.beefyInstantSyncer.Start(ctx, eg)
	if err != nil {
		return err
	}

	return nil
}
