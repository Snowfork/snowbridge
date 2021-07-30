package ethereum

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
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/ethereum"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"
)

var (
	configFile string
	substratePrivateKey string
	substratePrivateKeyFile string
)

func Command() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "ethereum",
		Short:   "Start the ethereum relay",
		Args:    cobra.ExactArgs(0),
		RunE:    runFunc,
	}

	cmd.Flags().StringVar(&configFile, "config", "", "Config file")
	cmd.Flags().StringVar(&substratePrivateKey, "private-key", "", "Private key URI for Substrate")
	cmd.Flags().StringVar(&substratePrivateKeyFile, "private-key-file", "", "The file from which to read the private key URI")

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

	var config ethereum.Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return nil
	}

	var privateKeyURI string

	if substratePrivateKey == "" {
		if substratePrivateKeyFile == "" {
			return fmt.Errorf("Private key URI not supplied")
		}
		contentBytes, err := ioutil.ReadFile(substratePrivateKeyFile)
		if err != nil {
			log.Fatal(err)
		}
		privateKeyURI = strings.TrimSpace(string(contentBytes))
	} else {
		privateKeyURI = substratePrivateKey
	}

	keypair, err := sr25519.NewKeypairFromSeed(privateKeyURI, 42)
	if err != nil {
		return fmt.Errorf("Unable to parse private key URI: %w", err)
	}

	relay := ethereum.NewRelay(&config, keypair)

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

	err = relay.Start(ctx, eg)
	if err != nil {
		return err
	}

	return eg.Wait()
}

