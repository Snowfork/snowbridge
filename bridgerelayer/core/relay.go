package core

import (
	"os"
	"os/signal"
	"syscall"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	chains []chain.Chain
}

func NewRelay(chains []chain.Chain) *Relay {
	return &Relay{chains}
}

func (re *Relay) Start() {

	for _, chain := range re.chains {
		err := chain.Start()
		if err != nil {
			log.WithFields(log.Fields{
				"chain": chain.Name(),
				"err":   err,
			}).Error("Failed to start chain")
			return
		}
		log.WithField("name", chain.Name()).Info("Started chain")
	}

	sigc := make(chan os.Signal, 1)
	signal.Notify(sigc, syscall.SIGINT, syscall.SIGTERM)
	defer signal.Stop(sigc)

	// Block here and wait for a signal
	select {
	case <-sigc:
		log.Info("Interrupt received, shutting down now.")
	}

	// Signal chains to shutdown
	for _, chain := range re.chains {
		chain.Stop()
	}
}
