package run

import (
	"github.com/snowfork/snowbridge/relayer/cmd/run/beacon"
	beaconstate "github.com/snowfork/snowbridge/relayer/cmd/run/beacon-state"
	"github.com/snowfork/snowbridge/relayer/cmd/run/beefy"
	ethereumv3 "github.com/snowfork/snowbridge/relayer/cmd/run/ethereum-v3"
	"github.com/snowfork/snowbridge/relayer/cmd/run/fisherman"
	parachainv3 "github.com/snowfork/snowbridge/relayer/cmd/run/parachain-v3"
	"github.com/snowfork/snowbridge/relayer/cmd/run/reward"
	"github.com/spf13/cobra"
)

func Command() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "run",
		Short: "Start a relay service",
		Args:  cobra.MinimumNArgs(1),
	}

	cmd.AddCommand(parachainv3.Command())
	cmd.AddCommand(beefy.Command())
	cmd.AddCommand(beacon.Command())
	cmd.AddCommand(beaconstate.Command())
	cmd.AddCommand(ethereumv3.Command())
	cmd.AddCommand(reward.Command())
	cmd.AddCommand(fisherman.Command())

	return cmd
}
