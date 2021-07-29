package cmd

import (
	"log"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/cmd/run"
	"github.com/spf13/cobra"
)


func runCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "run",
		Short:   "Start a relay service",
		Args:    cobra.MinimumNArgs(1),
		RunE:    RunFn,
	}

	cmd.AddCommand(run.BeefyCmd())

	return cmd
}

func RunFn(cmd *cobra.Command, _ []string) error {
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
