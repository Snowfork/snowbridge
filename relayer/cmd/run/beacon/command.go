package beacon

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"
)

var (
	configFile string
)

func Command() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "beacon",
		Short: "Start the beacon chain relay",
		Args:  cobra.ExactArgs(0),
		RunE:  run,
	}

	cmd.Flags().StringVar(&configFile, "config", "", "Path to configuration file")
	cmd.MarkFlagRequired("config")

	return cmd
}

func run(_ *cobra.Command, _ []string) error {
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.DebugLevel)

	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config beacon.Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return err
	}

	relay := beacon.NewRelay(&config)
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
