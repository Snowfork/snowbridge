package cmd

import (
	"fmt"
	"log"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func syncBeefyCommitmentCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sync-latest-beefy-commitment",
		Short: "Sync beefy commitment on demand",
		Args:  cobra.ExactArgs(0),
		RunE:  SyncBeefyCommitmentFn,
	}

	cmd.Flags().String("config", "/tmp/snowbridge/beefy-relay.json", "Path to configuration file")
	cmd.Flags().String("private-key", "", "Ethereum private key")
	cmd.Flags().String("private-key-file", "", "The file from which to read the private key")
	cmd.Flags().Uint64P("block-number", "b", 0, "Relay block number which contains a Parachain message")
	cmd.MarkFlagRequired("block-number")
	return cmd
}

func SyncBeefyCommitmentFn(cmd *cobra.Command, _ []string) error {
	ctx := cmd.Context()

	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.DebugLevel)

	configFile, err := cmd.Flags().GetString("config")
	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}

	var config beefy.Config
	err = viper.Unmarshal(&config)
	if err != nil {
		return err
	}
	privateKey, _ := cmd.Flags().GetString("private-key")
	privateKeyFile, _ := cmd.Flags().GetString("private-key-file")
	if privateKey == "" && privateKeyFile == "" {
		return fmt.Errorf("missing private key")
	}
	keypair, err := ethereum.ResolvePrivateKey(privateKey, privateKeyFile)
	if err != nil {
		return err
	}

	relay, err := beefy.NewRelay(&config, keypair)
	if err != nil {
		return err
	}
	blockNumber, _ := cmd.Flags().GetUint64("block-number")
	err = relay.OneShotSync(ctx, blockNumber)
	return err
}
