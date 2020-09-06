package cmd

import (
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/substrate"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
	"github.com/spf13/cobra"
)

func runCmd() *cobra.Command {
	//nolint:lll
	cmd := &cobra.Command{
		Use:     "run",
		Short:   "Relay messages between chains",
		Args:    cobra.ExactArgs(0),
		Example: "bridgerelayer run",
		RunE:    runFunc,
	}

	return cmd
}

func runFunc(_ *cobra.Command, _ []string) error {

	ethMessages := make(chan core.Message, 1)
	subMessages := make(chan core.Message, 1)

	ethChain, err := ethereum.NewChain(ethMessages, subMessages)
	if err != nil {
		return err
	}

	subChain, err := substrate.NewChain(ethMessages, subMessages)
	if err != nil {
		return err
	}

	relay := core.NewRelay(ethChain, subChain)

	relay.Start()

	return nil
}
