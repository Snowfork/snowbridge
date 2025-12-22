package cmd

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	para "github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/fisherman"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"
)

func reportEquivocationCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "report-equivocation",
		Short: "Report equivocation on demand",
		Args:  cobra.ExactArgs(0),
		RunE:  ReportEquivocationFn,
	}
	cmd.Flags().String("config", "", "Path to configuration file")
	cmd.MarkFlagRequired("config")

	cmd.Flags().String("ethereum.private-key", "", "Ethereum private key")
	cmd.Flags().String("ethereum.private-key-file", "", "The file from which to read the private key")
	cmd.Flags().String("ethereum.private-key-id", "", "The secret id to lookup the private key in AWS Secrets Manager")
	cmd.Flags().String("substrate.private-key", "", "substrate private key")

	cmd.Flags().Uint64("block-number", 1, "Ethereum block number")
	cmd.MarkFlagRequired("block-number")
	return cmd
}

func ReportEquivocationFn(cmd *cobra.Command, _ []string) error {
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.DebugLevel)

	configFile, err := cmd.Flags().GetString("config")
	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config fisherman.Config
	err = viper.UnmarshalExact(&config)
	if err != nil {
		return err
	}

	err = config.Validate()
	if err != nil {
		return fmt.Errorf("config file validation failed: %w", err)
	}

	ethereumPrivateKey, _ := cmd.Flags().GetString("ethereum.private-key")
	privateKeyFile, _ := cmd.Flags().GetString("ethereum.private-key-file")
	privateKeyID, _ := cmd.Flags().GetString("ethereum.private-key-id")
	keypair, err := ethereum.ResolvePrivateKey(ethereumPrivateKey, privateKeyFile, privateKeyID)
	if err != nil {
		return err
	}

	fishermanPrivateKey, _ := cmd.Flags().GetString("substrate.private-key")
	keypair2, err := para.ResolvePrivateKey(fishermanPrivateKey, "", "")
	if err != nil {
		return err
	}

	relay, err := fisherman.NewRelay(&config, keypair, keypair2)
	if err != nil {
		return err
	}

	blockNumber, _ := cmd.Flags().GetUint64("block-number")

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

	err = relay.Oneshot(ctx, eg, blockNumber)
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
