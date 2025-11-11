package fisherman

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	config            *Config
	relaychainConn    *relaychain.Connection
	ethereumConnBeefy *ethereum.Connection
	beefyListener     *BeefyListener
}

// TODO: the secp256k1 keypair is not used atm, but not refactoring out for now since we may use this
// with reactive fiat-shamir to thwart a subsampling attack
func NewRelay(config *Config, keypair *secp256k1.Keypair, keypair2 *sr25519.Keypair) (*Relay, error) {
	log.Info("Creating fisherman worker")

	relaychainWriterConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint, keypair2.AsKeyringPair())

	ethereumConnBeefy := ethereum.NewConnection(&config.Source.Ethereum, keypair)

	beefyListener := NewBeefyListener(
		&config.Source,
		ethereumConnBeefy,
		relaychainWriterConn,
	)

	return &Relay{
		config:            config,
		relaychainConn:    relaychainWriterConn,
		ethereumConnBeefy: ethereumConnBeefy,
		beefyListener:     beefyListener,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.ethereumConnBeefy.ConnectWithHeartBeat(ctx, eg, 30*time.Second)
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: beefy: %w", err)
	}

	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, eg, 30*time.Second)
	if err != nil {
		return err
	}

	log.Info("Starting beefy listener (fisherman)")
	err = relay.beefyListener.Start(ctx, eg)
	if err != nil {
		return err
	}

	log.Info("Starting equivocation fisherman")

	return nil
}

func (relay *Relay) Oneshot(ctx context.Context, eg *errgroup.Group, blockNumber uint64) error {
	err := relay.ethereumConnBeefy.ConnectWithHeartBeat(ctx, eg, 30*time.Second)
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: beefy: %w", err)
	}

	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, eg, 30*time.Second)
	if err != nil {
		return err
	}

	// Set up light client bridge contract
	address := common.HexToAddress(relay.beefyListener.config.Contracts.BeefyClient)
	beefyClientContract, err := contracts.NewBeefyClient(address, relay.ethereumConnBeefy.Client())
	if err != nil {
		return err
	}
	relay.beefyListener.beefyClientContract = beefyClientContract

	err = relay.beefyListener.checkSubmitInitialEquivocation(ctx, blockNumber)
	if err != nil {
		return fmt.Errorf("check submit initial equivocation: %w", err)
	}
	err = relay.beefyListener.checkSubmitFinalEquivocation(ctx, blockNumber)
	if err != nil {
		return fmt.Errorf("check submit final equivocation: %w", err)
	}

	log.Info("Oneshot equivocation reporting complete")

	return nil
}
