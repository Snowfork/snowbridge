package cmd

import (
	"fmt"
	_ "github.com/mattn/go-sqlite3"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func listBeaconStateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "list-beacon-states",
		Short: "List the beacon states in the beacon store.",
		Args:  cobra.ExactArgs(0),
		RunE:  listBeaconState,
	}

	cmd.Flags().String("config", "", "path to the beacon config file to use")
	err := cmd.MarkFlagRequired("config")

	if err != nil {
		return nil
	}

	return cmd
}

func listBeaconState(cmd *cobra.Command, _ []string) error {
	log.SetFormatter(&log.TextFormatter{
		DisableQuote: true, // so tab works in logs
	})

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

	p := protocol.New(conf.Source.Beacon.Spec, conf.Sink.Parachain.HeaderRedundancy)
	store := store.New(conf.Source.Beacon.DataStore.Location, conf.Source.Beacon.DataStore.MaxEntries, *p)

	err = store.Connect()
	if err != nil {
		return fmt.Errorf("connect to database: %w", err)
	}

	defer store.Close()

	states, err := store.ListBeaconStates()
	if err != nil {
		return err
	}

	log.WithField("count", len(states)).Info("found beacon states")

	log.Infof("| nr\t || ID\t | Finalized Slot | Attested Slot| Finalized Period\t | Attested Period\t | Attested File Exists? | Finalized File Exists? |")

	for i, state := range states {
		attestedFileExists := store.StateFileExists(state.AttestedStateFilename)
		finalizedFileExists := store.StateFileExists(state.AttestedStateFilename)
		log.Infof("| %d\t | %d\t | %d\t  | %d\t | %d\t\t\t | %d\t\t\t | %s\t\t\t | %s\t\t\t  |", i+1, state.ID, state.AttestedSlot, state.FinalizedSlot, state.FinalizedSyncPeriod, state.AttestedSyncPeriod, readableBool(attestedFileExists), readableBool(finalizedFileExists))
	}

	return nil
}

func readableBool(value bool) string {
	if value {
		return "yes"
	}
	return "no"
}
