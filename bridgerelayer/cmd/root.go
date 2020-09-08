package cmd

import (
	"fmt"
	"os"
	"path"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	homedir "github.com/mitchellh/go-homedir"
)

const userConfigDir = ".config/artemis-relayer"

var rootCmd = &cobra.Command{
	Use:          "bridgerelayer",
	Short:        "Relays data between Ethereum and Polkadot",
	SilenceUsage: true,
}

func init() {
	cobra.OnInitialize(loadConfig)

	rootCmd.AddCommand(runCmd())
}

func homeDir() string {
	home, err := homedir.Dir()
	if err != nil {
		fmt.Println("Error: ", err)
		os.Exit(1)
	}
	return home
}

func loadConfig() {

	home := homeDir()

	viper.AddConfigPath(path.Join(home, userConfigDir))
	viper.AddConfigPath(".")

	viper.SetConfigName("config")
	viper.SetConfigType("toml")

	viper.SetDefault("ethereum.registry-path", path.Join(home, userConfigDir, "ethereum"))

	err := viper.ReadInConfig()
	if err != nil {
		fmt.Println("Fatal error reading config file: ", err)
		os.Exit(1)
	}

	viper.BindEnv("ethereum.private-key", "ARTEMIS_RELAY_ETHEREUM_KEY")
	viper.BindEnv("substrate.private-key", "ARTEMIS_RELAY_SUBSTRATE_KEY")

	fmt.Println("Using config file:", viper.ConfigFileUsed())
}

// Execute adds all child commands to the root command
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
