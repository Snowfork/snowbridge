// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package cmd

import (
	"log"

	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
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

func RunFn(_ *cobra.Command, _ []string) error {
	setupLogging()

	relay, err := core.NewRelay()
	if err != nil {
		logrus.WithField("error", err).Error("Failed to initialize relayer")
		return err
	}

	relay.Start()

	return nil
}

func setupLogging() {
	logrus.SetLevel(logrus.DebugLevel)
	// Some of our dependencies such as GSRPC use the stdlib logger. So we need to
	// funnel those log messages into logrus.
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
}
