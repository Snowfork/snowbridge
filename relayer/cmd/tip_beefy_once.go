package cmd

import (
	"fmt"
	"log"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"
)

func tipBeefyOnceCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tip-beefy-once",
		Short: "One-shot BEEFY tip update (fiat-shamir by default; no Gateway messages)",
		Args:  cobra.ExactArgs(0),
		RunE:  TipBeefyOnceFn,
	}
	cmd.Flags().String("config", "", "Path to beefy configuration file")
	_ = cmd.MarkFlagRequired("config")
	cmd.Flags().String("private-key", "", "Ethereum private key")
	cmd.Flags().String("private-key-file", "", "The file from which to read the private key")
	cmd.Flags().String("private-key-id", "", "The secret id to lookup the private key in AWS Secrets Manager")
	cmd.Flags().Bool("two-phase", false, "Use two-phase submit instead of fiat-shamir")
	return cmd
}

func TipBeefyOnceFn(cmd *cobra.Command, _ []string) error {
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
	if err := viper.UnmarshalExact(&config); err != nil {
		return err
	}
	if err := config.Validate(); err != nil {
		return fmt.Errorf("config file validation failed: %w", err)
	}

	privateKey, _ := cmd.Flags().GetString("private-key")
	privateKeyFile, _ := cmd.Flags().GetString("private-key-file")
	privateKeyID, _ := cmd.Flags().GetString("private-key-id")
	keypair, err := ethereum.ResolvePrivateKey(privateKey, privateKeyFile, privateKeyID)
	if err != nil {
		return err
	}
	twoPhase, _ := cmd.Flags().GetBool("two-phase")

	relay, err := beefy.NewOnDemandRelay(&config, keypair)
	if err != nil {
		return err
	}

	eg, egCtx := errgroup.WithContext(ctx)
	if err := relay.InitializeOnDemandSync(egCtx, eg); err != nil {
		return err
	}

	before, err := relay.QueryBeefyClientState(egCtx)
	if err != nil {
		return err
	}
	tip, err := relay.RelayBeefyTip(egCtx)
	if err != nil {
		return err
	}
	logrus.WithFields(logrus.Fields{
		"lc":   before.LatestBeefyBlock,
		"tip":  tip,
		"set":  before.CurrentValidatorSetID,
		"next": before.NextValidatorSetID,
	}).Info("tip-beefy-once before")

	if err := relay.SyncNextTipFromLightClient(egCtx, twoPhase); err != nil {
		return err
	}

	after, err := relay.QueryBeefyClientState(egCtx)
	if err != nil {
		return err
	}
	logrus.WithFields(logrus.Fields{
		"lc":   after.LatestBeefyBlock,
		"set":  after.CurrentValidatorSetID,
		"next": after.NextValidatorSetID,
	}).Info("tip-beefy-once after")
	return nil
}
