// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package cmd

import (
	"log"

	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

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
	cmd.PersistentFlags().Int(
		"direction",
		0,
		"Relay messages bi-directionally (0), from Eth to Sub (1), or from Sub to Eth (2)",
	)
	cmd.PersistentFlags().Bool("headers-only", false, "Only forward headers")
	cmd.PersistentFlags().Bool("v2", false, "Use the new relayer")
	return cmd
}

func RunFn(cmd *cobra.Command, _ []string) error {
	setupLogging()

	// Bind flags that override their config file counterparts
	viper.BindPFlag("relay.direction", cmd.Flags().Lookup("direction"))
	viper.BindPFlag("relay.headers-only", cmd.Flags().Lookup("headers-only"))

	useV2 := cmd.Flags().Lookup("v2").Value.String() == "true"
	if useV2 {
		relay := &core.RelayV2{}
		return relay.Run()
	}

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
