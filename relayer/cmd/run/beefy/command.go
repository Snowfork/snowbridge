package beefy

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
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
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
		Use:   "beefy",
		Short: "Start the beefy relay",
		Args:  cobra.ExactArgs(0),
		RunE:  run,
	}

	cmd.Flags().StringVar(&configFile, "config", "", "Path to configuration file")
	cmd.MarkFlagRequired("config")

	cmd.Flags().StringVar(&privateKey, "ethereum.private-key", "", "Ethereum private key")
	cmd.Flags().StringVar(&privateKeyFile, "ethereum.private-key-file", "", "The file from which to read the private key")

	return cmd
}

func run(_ *cobra.Command, _ []string) error {
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.DebugLevel)

	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config beefy.Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return err
	}

	keypair, err := resolvePrivateKey(privateKey, privateKeyFile)
	if err != nil {
		return err
	}

	relay, err := beefy.NewRelay(&config, keypair)
	if err != nil {
		return err
	}

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

func resolvePrivateKey(privateKey, privateKeyFile string) (*secp256k1.Keypair, error) {
	var cleanedKey string

	if privateKey == "" {
		if privateKeyFile == "" {
			return nil, fmt.Errorf("private key not supplied")
		}
		contentBytes, err := ioutil.ReadFile(privateKeyFile)
		if err != nil {
			return nil, fmt.Errorf("failed to load private key: %w", err)
		}
		cleanedKey = strings.TrimPrefix(strings.TrimSpace(string(contentBytes)), "0x")
	} else {
		cleanedKey = strings.TrimPrefix(privateKey, "0x")
	}

	keypair, err := secp256k1.NewKeypairFromString(cleanedKey)
	if err != nil {
		return nil, fmt.Errorf("failed to parse private key: %w", err)
	}

	return keypair, nil
}
