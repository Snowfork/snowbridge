package cmd

import (
	"fmt"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"github.com/pkg/errors"
	"github.com/spf13/cobra"

	homedir "github.com/mitchellh/go-homedir"
	"github.com/spf13/viper"
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
		Use:     "init [polkadotRpcURL] [ethereumRpcUrl]",
		Short:   "Validate credentials and initialize subscriptions to both chains",
		Args:    cobra.ExactArgs(4),
		Example: "bridgerelayer init wss://rpc.polkadot.io wss://mainnet.infura.io/ws/v3",
		RunE:    RunInitRelayerCmd,
	}

	return initRelayerCmd
}

// RunInitRelayerCmd executes initRelayerCmd
func RunInitRelayerCmd(cmd *cobra.Command, args []string) error {
	// Validate and parse arguments
	if len(strings.Trim(args[0], "")) == 0 {
		return errors.Errorf("invalid [polkadot-rpc-url]: %s", args[0])
	}
	polkadotRpcUrl := args[0]

	if len(strings.Trim(args[0], "")) == 0 {
		return errors.Errorf("invalid [ethereum-rpc-url]: %s", args[0])
	}
	ethereumRpcUrl := args[1]

	// Init universial logger...
	// logger := _

	ethChain, err := ethereum.NewEthereumChain()
	if err != nil {
		return err
	}

	substrateChain, err := substrate.NewSubstrateChain()
	if err != nil {
		return err
	}

	go ethChain.Streamer.Start()
	go substrateChain.Streamer.Start()

	// Exit signal enables graceful shutdown
	exitSignal := make(chan os.Signal, 1)
	signal.Notify(exitSignal, syscall.SIGINT, syscall.SIGTERM)
	<-exitSignal

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
