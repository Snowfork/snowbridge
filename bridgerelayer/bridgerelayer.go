package main

import (
	"fmt"
	"os"

	"github.com/ethereum/go-ethereum/common"
	homedir "github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains"
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

//	initRelayerCmd
func initRelayerCmd() *cobra.Command {
	//nolint:lll
	initRelayerCmd := &cobra.Command{
		Use:     "init",
		Short:   "Validate credentials and initialize subscriptions to both chains",
		Args:    cobra.ExactArgs(0),
		Example: "bridgerelayer init",
		RunE:    RunInitRelayerCmd,
	}

	return initRelayerCmd
}

// RunInitRelayerCmd executes initRelayerCmd
func RunInitRelayerCmd(cmd *cobra.Command, args []string) error {
	// Load config
	config, err := chains.GetConfig()
	if err != nil {
		return err
	}
	c := *config

	// Set up individual chain configs
	var ethConfig chains.ChainConfig
	var subConfig chains.ChainConfig
	for _, chainConfig := range c.Chains {
		switch chainConfig.Type {
		case "ethereum":
			ethConfig = chainConfig
		case "substrate":
			subConfig = chainConfig
		default:
			return fmt.Errorf("invalid chain config type: %s", chainConfig.Type)
		}
	}

	// Initialize Ethereum chain
	ethStreamer := ethereum.NewStreamer(ethConfig.Endpoint)

	ethKeybase, err := eKeys.NewKeypairFromString(ethConfig.PrivateKey)
	if err != nil {
		return err
	}

	ethRouter, err := ethereum.NewRouter(ethConfig.Endpoint, ethKeybase, common.HexToAddress(ethConfig.Verifier))
	if err != nil {
		return err
	}

	ethChain := ethereum.NewEthChain(ethConfig, ethStreamer, *ethRouter)

	// Initialize Substrate chain
	_ = subConfig

	// Start chains
	ethChain.Start()
	// subChain.Start()

	return nil
}

func init() {
	cobra.OnInitialize(initConfig)

	// Persistent flags
	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.bridgerelayer.yaml)")

	// Construct Root Command
	rootCmd.AddCommand(
		initRelayerCmd(),
	)
}

// initConfig reads in config file and ENV variables if set.
func initConfig() {
	if cfgFile != "" {
		// Use config file from the flag.
		viper.SetConfigFile(cfgFile)
	} else {
		// Find home directory.
		home, err := homedir.Dir()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		// Search config in home directory with name ".bridgerelayer" (without extension).
		viper.AddConfigPath(home)
		viper.SetConfigName(".bridgerelayer")
	}

	viper.AutomaticEnv() // read in environment variables that match

	// If a config file is found, read it in.
	if err := viper.ReadInConfig(); err == nil {
		fmt.Println("Using config file:", viper.ConfigFileUsed())
	}
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
