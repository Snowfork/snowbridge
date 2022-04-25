package beefy

import (
	"context"
	"errors"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"
	"golang.org/x/sync/errgroup"
)

type Naka struct {
	sinkConfig SinkConfig,
	sourceConfig sourceConfig,
	ethConn *ethereum.Connection
	subConn *relaychain.Connection
	beefyLightClient beefylightclient.Contract
}

func NewNaka(
	sinkConfig SinkConfig,
	sourceConfig sourceConfig,
	ethConn *ethereum.Connection,
	subConn *relaychain.Connection,
) *EthereumWriter {
	return &Naka{
		sinkConfig, sourceConfig, ethConn, subConn,
	}
}

func (n *Naka) Start(ctx context.Context, eg errgroup.Group) error {
	address := common.HexToAddress(n.sinkConfig.Contracts.BeefyLightClient)
	contract, err := beefylightclient.NewContract(address, n.ethConn.GetClient())
	if err != nil {
		return 0, err
	}
	wr.beefyLightClient = contract

	eg.Go(func() error {
		err := n.watchNewSessionEvents(ctx)
		log.WithField("reason", err).Info("Shutting down NewSession event watcher")
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})
}

func (n *Naka) watchNewSessionEvents(ctx context.Context) error {
	opts := bind.WatchOpts{
		Context: ctx,
	}

	events := make(chan *beefylightclient.ContractNewSession)
	sub, err := n.beefyLightClient.WatchNewSession(&opts, events)
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-sub.Err():
			return err
		case _, ok := <-events:
			if !ok {
				return nil
			}
		}
	}
}
