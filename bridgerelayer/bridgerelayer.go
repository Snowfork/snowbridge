package main

import (
	"fmt"
	"os"

	"github.com/ethereum/go-ethereum/common"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/ethereum"
	eKeys "github.com/snowfork/polkadot-ethereum/bridgerelayer/keybase/ethereum"
	// "github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/substrate"
)

var cfgFile string

var rootCmd = &cobra.Command{
	Use:          "bridgerelayer",
	Short:        "Streams transactions from Ethereum and Polkadot and relays tx information to the opposite chain",
	SilenceUsage: true,
}

func runCmd() *cobra.Command {
	//nolint:lll
	cmd := &cobra.Command{
		Use:     "run",
		Short:   "Relaye messages between chains",
		Args:    cobra.ExactArgs(0),
		Example: "bridgerelayer run",
		RunE:    runFunc,
	}

	return cmd
}

func runFunc(cmd *cobra.Command, args []string) error {

	// Initialize Ethereum chain
	ethStreamer := ethereum.NewStreamer(viper.GetString("ethereum.endpoint"))

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

func init() {
	cobra.OnInitialize(loadConfig)

	rootCmd.AddCommand(runCmd())
}

func loadConfig() {

	viper.AddConfigPath("$HOME/.config/artemis-relayer")
	viper.SetConfigName("config")
	viper.SetConfigType("toml")
	viper.AutomaticEnv()

	err := viper.ReadInConfig()
	if err != nil {
		panic(fmt.Errorf("fatal error config file: %s", err))
	}

	fmt.Println("Using config file:", viper.ConfigFileUsed())
}

// Execute adds all child commands to the root command
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
