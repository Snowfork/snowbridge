package run

import (
	"github.com/snowfork/snowbridge/relayer/cmd/run/beacon"
	beaconstate "github.com/snowfork/snowbridge/relayer/cmd/run/beacon-state"
	"github.com/snowfork/snowbridge/relayer/cmd/run/beefy"
	"github.com/snowfork/snowbridge/relayer/cmd/run/ethereum"
	ethereumv2 "github.com/snowfork/snowbridge/relayer/cmd/run/ethereum-v2"
	"github.com/snowfork/snowbridge/relayer/cmd/run/fisherman"
	"github.com/snowfork/snowbridge/relayer/cmd/run/parachain"
	parachainv2 "github.com/snowfork/snowbridge/relayer/cmd/run/parachain-v2"
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
	cmd.AddCommand(parachainv2.Command())
	cmd.AddCommand(beacon.Command())
	cmd.AddCommand(beaconstate.Command())
	cmd.AddCommand(ethereum.Command())
	cmd.AddCommand(ethereumv2.Command())
	cmd.AddCommand(reward.Command())
	cmd.AddCommand(fisherman.Command())

	return cmd
}
