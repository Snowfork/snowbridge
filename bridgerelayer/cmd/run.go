package cmd

import (
	"fmt"
	"log"
	"os"
	"path"

	homedir "github.com/mitchellh/go-homedir"
	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
)

func runCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "run",
		Short:   "Start the relay service",
		Args:    cobra.ExactArgs(0),
		Example: "artemis-relay run",
		RunE:    RunFn,
	}
	return cmd
}

func RunFn(_ *cobra.Command, _ []string) error {

	loadConfig()
	setupLogging()

	relay, err := core.NewRelay()
	if err != nil {
		logrus.WithField("error", err).Error("Failed to initialize relayer")
		return err
	}

	relay.Start()

	return nil
}

func loadConfig() {
	home := homeDir()

	viper.AddConfigPath(path.Join(home, ".config", "artemis-relay"))
	viper.AddConfigPath(".")

	viper.SetConfigName("config")
	viper.SetConfigType("toml")

	viper.SetDefault("ethereum.registry-path", path.Join(home, ".config", "artemis-relay", "ethereum"))

	err := viper.ReadInConfig()
	if err != nil {
		fmt.Println("fatal error reading config file: ", err)
		os.Exit(1)
	}

	viper.BindEnv("ethereum.private-key", "ARTEMIS_RELAY_ETHEREUM_KEY")
	viper.BindEnv("substrate.private-key", "ARTEMIS_RELAY_SUBSTRATE_KEY")

	fmt.Println("Using config file:", viper.ConfigFileUsed())
}

func setupLogging() {
	logrus.SetLevel(logrus.DebugLevel)
	// Some of our dependencies such as GSRPC use the stdlib logger. So we need to
	// funnel those log messages into logrus.
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
}

func homeDir() string {
	home, err := homedir.Dir()
	if err != nil {
		fmt.Println("error: ", err)
		os.Exit(1)
	}
	return home
}
