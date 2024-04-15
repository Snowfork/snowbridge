package cmd

import (
	"fmt"
	log "github.com/sirupsen/logrus"
	"strconv"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	_ "github.com/mattn/go-sqlite3"
)

func storeBeaconState() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "store-beacon-state",
		Short: "Import the provided execution header.",
		Args:  cobra.ExactArgs(0),
		RunE:  storeBeaconStateInDB,
	}

	cmd.Flags().String("config", "", "path to the beacon config file to use")
	err := cmd.MarkFlagRequired("config")
	if err != nil {
		return nil
	}

	return cmd
}

func storeBeaconStateInDB(cmd *cobra.Command, _ []string) error {
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

	store := store.New(conf.Source.Beacon.DataStore.Location, conf.Source.Beacon.DataStore.MaxEntries)

	specSettings := conf.Source.Beacon.Spec
	beaconClient := api.NewBeaconClient(conf.Source.Beacon.Endpoint, specSettings.SlotsInEpoch)
	syncer := syncer.New(beaconClient, specSettings, &store)

	err = store.Connect()
	if err != nil {
		return err
	}

	defer store.Close()

	update, err := syncer.GetFinalizedUpdate()
	if err != nil {
		return err
	}

	attestedHeaderSlot := uint64(update.Payload.AttestedHeader.Slot)
	finalizedHeaderSlot := uint64(update.Payload.FinalizedHeader.Slot)
	attestedSyncPeriod := syncer.ComputeSyncPeriodAtSlot(attestedHeaderSlot)
	finalizedSyncPeriod := syncer.ComputeSyncPeriodAtSlot(finalizedHeaderSlot)

	attestedBeaconData, err := syncer.Client.GetBeaconState(strconv.FormatUint(attestedHeaderSlot, 10))
	if err != nil {
		return fmt.Errorf("download attested beacon state at slot %d: %w", attestedHeaderSlot, err)
	}
	finalizedBeaconData, err := syncer.Client.GetBeaconState(strconv.FormatUint(finalizedHeaderSlot, 10))
	if err != nil {
		return fmt.Errorf("download finalized beacon state at slot %d: %w", finalizedHeaderSlot, err)
	}

	err = store.WriteStateFile(attestedHeaderSlot, attestedBeaconData)
	if err != nil {
		return err
	}
	err = store.WriteStateFile(finalizedHeaderSlot, finalizedBeaconData)
	if err != nil {
		return err
	}

	err = store.StoreUpdate(attestedHeaderSlot, finalizedHeaderSlot, attestedSyncPeriod, finalizedSyncPeriod)
	if err != nil {
		return fmt.Errorf("store beacon update: %w", err)
	}

	deletedSlots, err := store.PruneOldStates()
	log.WithField("deletedSlots", deletedSlots).Info("deleted old beacon states")

	return nil
}
