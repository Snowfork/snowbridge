package cmd

import (
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/substrate"
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

	ethChain, err := ethereum.NewChain()
	if err != nil {
		return err
	}

	subChain, err := substrate.NewChain()
	if err != nil {
		return err
	}

	go ethChain.Start()
	go subChain.Start()

	return nil
}
