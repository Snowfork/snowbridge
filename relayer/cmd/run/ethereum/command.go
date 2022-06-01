package ethereum

import (
	"context"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/ethereum"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"
)

var (
	configFile     string
	privateKey     string
	privateKeyFile string
)

func Command() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "ethereum",
		Short: "Start the ethereum relay",
		Args:  cobra.ExactArgs(0),
		RunE:  run,
	}

	cmd.Flags().StringVar(&configFile, "config", "", "Path to configuration file")
	cmd.MarkFlagRequired("config")

	cmd.Flags().StringVar(&privateKey, "substrate.private-key", "", "Private key URI for Substrate")
	cmd.Flags().StringVar(&privateKeyFile, "substrate.private-key-file", "", "The file from which to read the private key URI")

	return cmd
}

func run(_ *cobra.Command, _ []string) error {
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.DebugLevel)

	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config ethereum.Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return err
	}

	keypair, err := resolvePrivateKey(privateKey, privateKeyFile)
	if err != nil {
		return err
	}

	relay := ethereum.NewRelay(&config, keypair)
	if err != nil {
		return err
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

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
		logrus.WithError(err).Fatal("Unhandled error")
		cancel()
		return err
	}

	err = eg.Wait()
	if err != nil {
		logrus.WithError(err).Fatal("Unhandled error")
		return err
	}

	return nil
}

func resolvePrivateKey(privateKey, privateKeyFile string) (*sr25519.Keypair, error) {
	var cleanedKeyURI string

	if privateKey == "" {
		if privateKeyFile == "" {
			return nil, fmt.Errorf("Private key URI not supplied")
		}
		content, err := ioutil.ReadFile(privateKeyFile)
		if err != nil {
			log.Fatal(err)
		}
		cleanedKeyURI = strings.TrimSpace(string(content))
	} else {
		cleanedKeyURI = privateKey
	}

	keypair, err := sr25519.NewKeypairFromSeed(cleanedKeyURI, 42)
	if err != nil {
		return nil, fmt.Errorf("Unable to parse private key URI: %w", err)
	}

	return keypair, nil
}
