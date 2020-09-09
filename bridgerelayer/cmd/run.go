package cmd

import (
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
	"github.com/spf13/cobra"

	log "github.com/sirupsen/logrus"
)

func runCmd() *cobra.Command {
	//nolint:lll
	cmd := &cobra.Command{
		Use:     "run",
		Short:   "Start the relay service",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay run",
		RunE:    runFunc,
	}

	return cmd
}

func runFunc(_ *cobra.Command, _ []string) error {

	log.SetLevel(log.DebugLevel)

	relay, err := core.NewRelay()
	if err != nil {
		log.WithField("error", err).Error("Failed to initialize relayer")
		return err
	}

	relay.Start()

	return nil
}
