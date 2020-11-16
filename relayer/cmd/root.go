// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package cmd

import (
	"fmt"
	"os"
	"path"

	"github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var configFile string

var rootCmd = &cobra.Command{
	Use:          "artemis-relay",
	Short:        "Artemis Relay is a bridge between Ethereum and Polkadot",
	SilenceUsage: true,
}

func init() {
	cobra.OnInitialize(initConfig)
	rootCmd.PersistentFlags().StringVar(&configFile, "config", "", "config file")
	rootCmd.PersistentFlags().Int(
		"direction",
		0,
		"Relay messages bi-directionally (0), from Eth to Sub (1), or from Sub to Eth (2)",
	)
	rootCmd.PersistentFlags().Bool("headers", false, "Only forward headers")
	rootCmd.AddCommand(runCmd())
}

func initConfig() {

	if configFile != "" {
		viper.SetConfigFile(configFile)
	} else {
		// Find home directory.
		home, err := homedir.Dir()
		if err != nil {
			fmt.Println("Error: ", err)
			os.Exit(1)
		}

		viper.AddConfigPath(path.Join(home, ".config", "artemis-relay"))
		viper.AddConfigPath(".")
		viper.SetConfigName("config")
		viper.SetConfigType("toml")
	}

	// Bind flags that override their config file counterparts
	viper.BindPFlag("relay.direction", rootCmd.Flags().Lookup("direction"))
	viper.BindPFlag("relay.headers-only", rootCmd.Flags().Lookup("headers"))

	if err := viper.ReadInConfig(); err == nil {
		fmt.Println("Using config file:", viper.ConfigFileUsed())
	} else {
		fmt.Println("Error: ", err)
		os.Exit(1)
	}
}

// Execute adds all child commands to the root command
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
