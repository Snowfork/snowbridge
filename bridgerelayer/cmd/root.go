package cmd

import (
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:          "artemis-relay",
	Short:        "Artemis Relay is a bridge between Ethereum and Polkadot",
	SilenceUsage: true,
}

func init() {
	rootCmd.AddCommand(runCmd())
}

// Execute adds all child commands to the root command
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
