package beacon

import (
	"context"
	"fmt"
	"github.com/ethereum/go-ethereum/rpc"
	"time"

	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	beaconstate "github.com/snowfork/snowbridge/relayer/relays/beacon-state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"

	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config  *config.Config
	keypair *sr25519.Keypair
}

func NewRelay(
	config *config.Config,
	keypair *sr25519.Keypair,
) *Relay {
	return &Relay{
		config:  config,
		keypair: keypair,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	specSettings := r.config.Source.Beacon.Spec
	log.WithField("spec", specSettings).Info("spec settings")

	paraconn := parachain.NewConnection(r.config.Sink.Parachain.ParseEndpoints(), r.keypair.AsKeyringPair())

	err := paraconn.ConnectWithHeartBeat(ctx, eg, time.Second*time.Duration(r.config.Sink.Parachain.HeartbeatSecs))
	if err != nil {
		return err
	}

	writer := parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
	)

	p := protocol.New(specSettings, r.config.Sink.Parachain.HeaderRedundancy)

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	beaconAPI := api.NewBeaconClient(r.config.Source.Beacon.ParseEndpoints())

	var stateServiceClient syncer.StateServiceClient
	if r.config.Source.Beacon.StateServiceEndpoint != "" {
		stateServiceClient = beaconstate.NewClient(r.config.Source.Beacon.StateServiceEndpoint)
		log.WithField("endpoint", r.config.Source.Beacon.StateServiceEndpoint).Info("Using beacon state service for proof generation")
	}

	headers := header.New(
		writer,
		beaconAPI,
		specSettings,
		p,
		r.config.Sink.UpdateSlotInterval,
		stateServiceClient,
	)

	// Gloas proofs need the execution header, which the beacon block no longer carries. The
	// config is validated to require this endpoint once Gloas is scheduled.
	if endpoint := r.config.Source.Execution.Endpoint; endpoint != "" {
		executionClient, err := rpc.DialContext(ctx, endpoint)
		if err != nil {
			return fmt.Errorf("dial execution endpoint: %w", err)
		}
		defer executionClient.Close()
		headers = headers.WithExecutionClient(executionClient)
		log.WithField("endpoint", endpoint).Info("Using execution endpoint for gloas execution proofs")
	}

	return headers.Sync(ctx, eg)
}
