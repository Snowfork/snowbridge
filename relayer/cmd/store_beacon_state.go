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

	cmd.Flags().String("url", "", "URL to generate test fixtures from")
	err := cmd.MarkFlagRequired("url")
	if err != nil {
		return nil
	}

	cmd.Flags().String("db-store-location", "", "where the database store file should be stored")
	err = cmd.MarkFlagRequired("db-store-location")
	if err != nil {
		return nil
	}

	return cmd
}

func storeBeaconStateInDB(cmd *cobra.Command, _ []string) error {
	dbStoreLocation, err := cmd.Flags().GetString("db-store-location")
	if err != nil {
		return err
	}

	url, err := cmd.Flags().GetString("url")
	if err != nil {
		return err
	}

	viper.SetConfigFile("web/packages/test/config/beacon-relay.json")
	if err := viper.ReadInConfig(); err != nil {
		return err
	}
	var conf config.Config
	err = viper.Unmarshal(&conf)
	if err != nil {
		return err
	}

	store := store.New(dbStoreLocation, 100)

	specSettings := conf.Source.Beacon.Spec
	beaconClient := api.NewBeaconClient(url, specSettings.SlotsInEpoch)
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
