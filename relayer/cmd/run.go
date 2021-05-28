package cmd

import (
	"log"

	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"

	"github.com/snowfork/polkadot-ethereum/relayer/core"
)

func runCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "run",
		Short:   "Start the relay service",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay run",
		RunE:    RunFn,
	}
	return cmd
}

func RunFn(cmd *cobra.Command, _ []string) error {
	setupLogging()

	relay := &core.Relay{}
	return relay.Run()
}

func setupLogging() {
	logrus.SetLevel(logrus.DebugLevel)
	// Some of our dependencies such as GSRPC use the stdlib logger. So we need to
	// funnel those log messages into logrus.
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
}
