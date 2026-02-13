package run

import (
	"github.com/snowfork/snowbridge/relayer/cmd/run/beacon"
	"github.com/snowfork/snowbridge/relayer/cmd/run/beefy"
	"github.com/snowfork/snowbridge/relayer/cmd/run/execution"
	executionv1 "github.com/snowfork/snowbridge/relayer/cmd/run/execution-v1"
	"github.com/snowfork/snowbridge/relayer/cmd/run/fisherman"
	"github.com/snowfork/snowbridge/relayer/cmd/run/parachain"
	parachainv1 "github.com/snowfork/snowbridge/relayer/cmd/run/parachain-v1"
	"github.com/snowfork/snowbridge/relayer/cmd/run/reward"
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
	cmd.AddCommand(beacon.Command())
	cmd.AddCommand(execution.Command())
	cmd.AddCommand(reward.Command())
	cmd.AddCommand(fisherman.Command())
	cmd.AddCommand(executionv1.Command())
	cmd.AddCommand(parachainv1.Command())

	return cmd
}
