package core

import (
	"context"
	"os"
	"os/signal"
	"syscall"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	chains []chain.Chain
}

func NewRelay(ethChain chain.Chain, subChain chain.Chain) *Relay {
	return &Relay{
		chains: []chain.Chain{ethChain, subChain},
	}
}

func (re *Relay) Start() {

	// Ensure clean termination upon SIGINT, SIGTERM
	ctx, cancel := context.WithCancel(context.Background())
	go func() {
		notify := make(chan os.Signal, 1)
		signal.Notify(notify, syscall.SIGINT, syscall.SIGTERM)
		<-notify
		log.Info("Received signal and terminating cleanly")
		cancel()
	}()

	eg, ctx := errgroup.WithContext(ctx)

	for _, chain := range re.chains {
		err := chain.Start(ctx, eg)
		if err != nil {
			log.WithFields(log.Fields{
				"chain": chain.Name(),
				"error": err,
			}).Error("Failed to start chain")
			return
		}
		log.WithField("name", chain.Name()).Info("Started chain")
	}

	// Wait until a fatal error or signal is raised
	if err := eg.Wait(); err != nil {
		if err != context.Canceled {
			log.WithFields(log.Fields{
				"error": err,
			}).Error("Encountered a fatal error")
		}
	}

	// Shutdown chains
	for _, chain := range re.chains {
		chain.Stop()
	}
}
