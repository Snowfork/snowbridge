package beefy

import (
	"context"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/signal"
	"path"
	"strings"
	"syscall"

	"github.com/mitchellh/go-homedir"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"
)

var (
	configFile string
	ethereumPrivateKey string
	ethereumPrivateKeyFile string
)

func BeefyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "beefy",
		Short:   "Start the beefy relay",
		Args:    cobra.ExactArgs(0),
		RunE:    runFunc,
	}

	cmd.Flags().StringVar(&configFile, "config", "", "Config file")
	cmd.Flags().StringVar(&ethereumPrivateKey, "private-key", "", "Ethereum private key")
	cmd.Flags().StringVar(&ethereumPrivateKeyFile, "private-key-file", "", "The file from which to read the private key")

	return cmd
}

func runFunc(_ *cobra.Command, _ []string) error {
	// Logging
	logrus.SetLevel(logrus.DebugLevel)
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))

	// Configuration
	if configFile != "" {
		viper.SetConfigFile(configFile)
	} else {
		home, err := homedir.Dir()
		if err != nil {
			return err
		}

		viper.AddConfigPath(".")
		viper.AddConfigPath(path.Join(home, ".config", "snowbridge-relay"))
		viper.AddConfigPath("/etc/snowbridge-relay")

		viper.SetConfigName("beefy")
		viper.SetConfigType("toml")
	}

	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config beefy.Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return nil
	}

	var privateKey string

	if ethereumPrivateKey == "" {
		if ethereumPrivateKeyFile == "" {
			return fmt.Errorf("Ethereum private key not supplied")
		}
		contentBytes, err := ioutil.ReadFile(ethereumPrivateKeyFile)
		if err != nil {
			log.Fatal(err)
		}
		privateKey = strings.TrimPrefix(strings.TrimSpace(string(contentBytes)), "0x")
	} else {
		privateKey = strings.TrimPrefix(ethereumPrivateKey, "0x")
	}

	keypair, err := secp256k1.NewKeypairFromString(privateKey)
	if err != nil {
		return fmt.Errorf("Unable to parse private key: %w", err)
	}

	relay, err := beefy.NewRelay(&config, keypair)

	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)

	// Ensure clean termination upon SIGINT, SIGTERM
	eg.Go(func() error {
		notify := make(chan os.Signal, 1)
		signal.Notify(notify, syscall.SIGINT, syscall.SIGTERM)

		select {
		case <-ctx.Done():
			return ctx.Err()
		case sig := <-notify:
			logrus.WithField("signal", sig.String()).Info("Received signal")
			cancel()
		}

		return nil
	})

	relay.Start(ctx, eg)

	return eg.Wait()
}

