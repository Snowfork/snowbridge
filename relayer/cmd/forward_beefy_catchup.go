package cmd

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"
)

func forwardBeefyCatchupCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "forward-beefy-catchup",
		Short: "Catch up BEEFY light client session-by-session (forward scan from LC tip)",
		Args:  cobra.ExactArgs(0),
		RunE:  ForwardBeefyCatchupFn,
	}
	cmd.Flags().String("config", "/tmp/snowbridge-v2/beefy-relay.json", "Path to beefy configuration file")
	cmd.Flags().String("private-key", "", "Ethereum private key")
	cmd.Flags().String("private-key-file", "", "The file from which to read the private key")
	cmd.Flags().String("private-key-id", "", "The secret id to lookup the private key in AWS Secrets Manager")
	cmd.Flags().Uint64("max-steps", 64, "Max sync steps before stopping")
	cmd.Flags().Uint64("target-lag", 512, "Stop when relay tip - LC tip is at or below this many blocks")
	return cmd
}

func ForwardBeefyCatchupFn(cmd *cobra.Command, _ []string) error {
	ctx := cmd.Context()

	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.InfoLevel)

	configFile, err := cmd.Flags().GetString("config")
	if err != nil {
		return err
	}
	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config beefy.Config
	if err := viper.Unmarshal(&config); err != nil {
		return err
	}

	privateKey, _ := cmd.Flags().GetString("private-key")
	privateKeyFile, _ := cmd.Flags().GetString("private-key-file")
	privateKeyID, _ := cmd.Flags().GetString("private-key-id")
	keypair, err := ethereum.ResolvePrivateKey(privateKey, privateKeyFile, privateKeyID)
	if err != nil {
		return err
	}

	maxSteps, _ := cmd.Flags().GetUint64("max-steps")
	targetLag, _ := cmd.Flags().GetUint64("target-lag")

	relay, err := beefy.NewOnDemandRelay(&config, keypair)
	if err != nil {
		return err
	}

	eg, egCtx := errgroup.WithContext(ctx)
	if err := relay.InitializeOnDemandSync(egCtx, eg); err != nil {
		return err
	}

	for step := uint64(1); step <= maxSteps; step++ {
		state, err := relay.QueryBeefyClientState(egCtx)
		if err != nil {
			return err
		}
		tip, err := relay.RelayBeefyTip(egCtx)
		if err != nil {
			return err
		}
		lag := uint64(0)
		if tip > state.LatestBeefyBlock {
			lag = tip - state.LatestBeefyBlock
		}
		logrus.WithFields(logrus.Fields{
			"step": step,
			"lc":   state.LatestBeefyBlock,
			"tip":  tip,
			"lag":  lag,
			"set":  state.CurrentValidatorSetID,
			"next": state.NextValidatorSetID,
		}).Info("Catch-up loop status")

		if lag <= targetLag {
			logrus.Info("Target lag reached; stopping")
			return nil
		}

		var stepErr error
		for attempt := 1; attempt <= 5; attempt++ {
			stepErr = relay.SyncNextFromLightClient(egCtx)
			if stepErr == nil {
				break
			}
			// ethclient / JSON-RPC wrappers rarely expose typed rate-limit or
			// disconnect errors; substring match is intentional for this ops CLI.
			if !isTransientRPCError(stepErr) {
				return fmt.Errorf("step %d failed: %w", step, stepErr)
			}
			logrus.WithError(stepErr).WithField("attempt", attempt).Warn("Transient ETH error; rotating endpoint and retrying")
			relay.RotateEthereumEndpoint(egCtx)
			select {
			case <-egCtx.Done():
				return egCtx.Err()
			case <-time.After(time.Duration(attempt*15) * time.Second):
			}
		}
		if stepErr != nil {
			return fmt.Errorf("step %d failed after retries: %w", step, stepErr)
		}

		// Allow chain / mempool to settle between sessions.
		select {
		case <-egCtx.Done():
			return egCtx.Err()
		case <-time.After(20 * time.Second):
		}
	}

	logrus.Warn("Max steps reached; light client may still be behind")
	return nil
}

func isTransientRPCError(err error) bool {
	if err == nil {
		return false
	}
	errText := err.Error()
	return strings.Contains(errText, "429") ||
		strings.Contains(errText, "Too Many Requests") ||
		strings.Contains(errText, "rate limit") ||
		strings.Contains(errText, "timeout") ||
		strings.Contains(errText, "EOF") ||
		strings.Contains(errText, "connection reset")
}
