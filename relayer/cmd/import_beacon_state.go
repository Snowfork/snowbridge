package cmd

import (
	"fmt"
	"os"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"

	_ "github.com/mattn/go-sqlite3"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func importBeaconStateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "import-beacon-state",
		Short: "Import the provided attested and finalized beacon states.",
		Args:  cobra.ExactArgs(0),
		RunE:  importBeaconState,
	}

	cmd.Flags().String("config", "", "path to the beacon config file to use")
	err := cmd.MarkFlagRequired("config")
	cmd.Flags().String("attested-state-file", "", "path to attested state file")
	err = cmd.MarkFlagRequired("finalized-state-file")
	cmd.Flags().String("finalized-state-file", "", "path to finalized state file")
	err = cmd.MarkFlagRequired("finalized-state-file")
	if err != nil {
		return nil
	}

	return cmd
}

func importBeaconState(cmd *cobra.Command, _ []string) error {
	configFile, err := cmd.Flags().GetString("config")
	if err != nil {
		return err
	}

	viper.SetConfigFile(configFile)
	if err := viper.ReadInConfig(); err != nil {
		return err
	}
	var conf config.Config
	err = viper.Unmarshal(&conf)
	if err != nil {
		return err
	}

	attestedStateFilePath, err := cmd.Flags().GetString("attested-state-file")
	if err != nil {
		return err
	}
	finalizedStateFilePath, err := cmd.Flags().GetString("finalized-state-file")
	if err != nil {
		return err
	}

	_, err = os.Stat(attestedStateFilePath)
	if err != nil {
		return fmt.Errorf("open attested state file: %w", err)
	}
	_, err = os.Stat(finalizedStateFilePath)
	if err != nil {
		return fmt.Errorf("open finalized state file: %w", err)
	}

	p := protocol.New(conf.Source.Beacon.Spec, conf.Sink.Parachain.HeaderRedundancy)
	store := store.New(conf.Source.Beacon.DataStore.Location, conf.Source.Beacon.DataStore.MaxEntries, *p)
	beaconClient := api.NewBeaconClient(conf.Source.Beacon.Endpoint, conf.Source.Beacon.StateEndpoint)
	syncer := syncer.New(beaconClient, &store, p)

	err = store.Connect()
	if err != nil {
		return fmt.Errorf("connect to database: %w", err)
	}

	defer store.Close()

	attestedData, err := os.ReadFile(attestedStateFilePath)
	if err != nil {
		return fmt.Errorf("read attested state data from file: %w", err)
	}
	finalizedData, err := os.ReadFile(finalizedStateFilePath)
	if err != nil {
		return fmt.Errorf("read finalized state data from file: %w", err)
	}

	afterDenebFork := (conf.Source.Beacon.Spec.ForkVersions.Deneb + 1) * 32

	attestedState, err := syncer.UnmarshalBeaconState(afterDenebFork, attestedData)
	if err != nil {
		return fmt.Errorf("unmarshal attested beacon state: %w", err)
	}
	finalizedState, err := syncer.UnmarshalBeaconState(afterDenebFork, finalizedData)
	if err != nil {
		return fmt.Errorf("unmarshal finalized beacon state: %w", err)
	}

	attestedSlot := attestedState.GetSlot()
	finalizedSlot := finalizedState.GetSlot()

	err = syncer.ValidatePair(finalizedSlot, attestedSlot, attestedState)
	if err != nil {
		return fmt.Errorf("state pair validation failed: %w", err)
	}

	err = store.WriteEntry(attestedSlot, finalizedSlot, attestedData, finalizedData)
	if err != nil {
		return fmt.Errorf("write beacon store entry: %w", err)
	}

	log.WithFields(log.Fields{"attestedSlot": attestedSlot, "finalizedSlot": finalizedSlot}).Info("imported beacon state")

	return nil
}
