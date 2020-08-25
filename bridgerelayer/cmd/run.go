package cmd

import (
	"fmt"
	"os"
	"path"

	"github.com/ethereum/go-ethereum/common"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	homedir "github.com/mitchellh/go-homedir"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/ethereum"
	eKeys "github.com/snowfork/polkadot-ethereum/bridgerelayer/keybase/ethereum"
	// "github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/substrate"
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

func registryPath() string {

	home, err := homedir.Dir()
	if err != nil {
		fmt.Println("Error:", err)
		os.Exit(1)
	}

	return path.Join(home, configDir, "ethereum")
}

func runFunc(cmd *cobra.Command, args []string) error {

	// Load ethereum ABIs

	ethStreamer := ethereum.NewStreamer(viper.GetString("ethereum.endpoint"), registryPath())

	ethKeybase, err := eKeys.NewKeypairFromString(viper.GetString("ethereum.private_key"))
	if err != nil {
		return err
	}

	ethRouter, err := ethereum.NewRouter(viper.GetString("ethereum.endpoint"), ethKeybase, common.HexToAddress(viper.GetString("ethereum.verifier")))
	if err != nil {
		return err
	}

	ethChain := ethereum.NewEthChain(ethStreamer, *ethRouter)

	// Start chains
	ethChain.Start()
	// subChain.Start()

	return nil
}
