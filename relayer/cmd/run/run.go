package run

import (
	"github.com/snowfork/snowbridge/relayer/cmd/run/beefy"
	"github.com/snowfork/snowbridge/relayer/cmd/run/ethereum"
	"github.com/snowfork/snowbridge/relayer/cmd/run/parachain"
	"github.com/spf13/cobra"
)

func Command() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "run",
		Short: "Start a relay service",
		Args:  cobra.MinimumNArgs(1),
	}

	cmd.AddCommand(beefy.Command())
	cmd.AddCommand(parachain.Command())
	cmd.AddCommand(ethereum.Command())

	return cmd
}
