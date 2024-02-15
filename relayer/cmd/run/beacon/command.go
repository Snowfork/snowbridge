package beacon

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/beacon"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
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
		Use:   "beacon",
		Short: "Start the beacon chain relay",
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

	logrus.Info("Beacon relayer started up")

	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config config.Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return err
	}

	keypair, err := parachain.ResolvePrivateKey(privateKey, privateKeyFile)
	if err != nil {
		return err
	}

	relay := beacon.NewRelay(&config, keypair)
	if err != nil {
		return err
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	eg, ctx := errgroup.WithContext(ctx)

	eg.Go(func() error {
		return startHttpServer(ctx)
	})

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

func startHttpServer(ctx context.Context) error {
	srv := &http.Server{Addr: ":8080", Handler: nil}

	http.HandleFunc("/beacon/health", healthCheckHandler)

	go func() {
		<-ctx.Done()
		if err := srv.Shutdown(context.Background()); err != nil {
			logrus.WithError(err).Error("beacon health check endpoint shutting down")
		}
	}()

	logrus.Info("starting health check HTTP server on port 8080")
	if err := srv.ListenAndServe(); err != http.ErrServerClosed {
		return fmt.Errorf("http server could not be started: %v", err)
	}

	return nil
}

func healthCheckHandler(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
}
