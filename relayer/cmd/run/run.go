package run

import (
	"log"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/cmd/run/beefy"
	"github.com/snowfork/snowbridge/relayer/cmd/run/ethereum"
	"github.com/snowfork/snowbridge/relayer/cmd/run/parachain"
	"github.com/spf13/cobra"
)


func Command() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "run",
		Short:   "Start a relay service",
		Args:    cobra.MinimumNArgs(1),
		RunE:    RunFn,
	}

	cmd.AddCommand(beefy.Command())
	cmd.AddCommand(parachain.Command())
	cmd.AddCommand(ethereum.Command())

	return cmd
}

func RunFn(_ *cobra.Command, _ []string) error {
	setupLogging()

	//relay := &core.Relay{}
	return nil
}

func setupLogging() {
	logrus.SetLevel(logrus.DebugLevel)
	// Some of our dependencies such as GSRPC use the stdlib logger. So we need to
	// funnel those log messages into logrus.
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
}
