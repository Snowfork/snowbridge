package cmd

import (
	"fmt"
	"os"
	"path"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

const configDir = ".config/artemis-relayer"

var rootCmd = &cobra.Command{
	Use:          "bridgerelayer",
	Short:        "Streams transactions from Ethereum and Polkadot and relays tx information to the opposite chain",
	SilenceUsage: true,
}

func init() {
	cobra.OnInitialize(loadConfig)
	rootCmd.AddCommand(runCmd())
}

func loadConfig() {

	viper.AddConfigPath(path.Join("$HOME", configDir))
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
