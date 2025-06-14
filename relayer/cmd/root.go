// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package cmd

import (
	"os"

	"github.com/snowfork/snowbridge/relayer/cmd/run"
	"github.com/spf13/cobra"
)

var dataDir string
var configFile string

var rootCmd = &cobra.Command{
	Use:          "snowbridge-relay",
	Short:        "Snowbridge Relay is a bridge between Ethereum and Polkadot",
	SilenceUsage: true,
}

func init() {
	rootCmd.AddCommand(run.Command())
	rootCmd.AddCommand(getBlockCmd())
	rootCmd.AddCommand(subBeefyCmd())
	rootCmd.AddCommand(scanBeefyCmd())
	rootCmd.AddCommand(scanSingleBeefyBlockCmd())
	rootCmd.AddCommand(leafCmd())
	rootCmd.AddCommand(basicChannelLeafProofCmd())
	rootCmd.AddCommand(parachainHeadProofCmd())
	rootCmd.AddCommand(importExecutionHeaderCmd())
	rootCmd.AddCommand(generateBeaconFixtureCmd())
	rootCmd.AddCommand(generateBeaconCheckpointCmd())
	rootCmd.AddCommand(generateExecutionUpdateCmd())
	rootCmd.AddCommand(generateInboundFixtureCmd())
	rootCmd.AddCommand(storeBeaconStateCmd())
	rootCmd.AddCommand(importBeaconStateCmd())
	rootCmd.AddCommand(listBeaconStateCmd())
	rootCmd.AddCommand(generateDeliveryProofFixtureCmd())
	rootCmd.AddCommand(syncBeefyCommitmentCmd())
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
