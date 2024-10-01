package cmd

import (
	"fmt"
	"strconv"

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

func storeBeaconStateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "store-beacon-state",
		Short: "Download and store the latest finalized beacon states",
		Args:  cobra.ExactArgs(0),
		RunE:  storeBeaconState,
	}

	cmd.Flags().String("config", "", "path to the beacon config file to use")
	err := cmd.MarkFlagRequired("config")
	if err != nil {
		return nil
	}

	return cmd
}

func storeBeaconState(cmd *cobra.Command, _ []string) error {
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
	beaconClient := api.NewBeaconClient(conf.Source.Beacon.Endpoint, conf.Source.Beacon.StateEndpoint)
	syncer := syncer.New(beaconClient, &store, p)

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

	attestedBeaconData, err := syncer.Client.GetBeaconState(strconv.FormatUint(attestedHeaderSlot, 10))
	if err != nil {
		return fmt.Errorf("download attested beacon state at slot %d: %w", attestedHeaderSlot, err)
	}
	finalizedBeaconData, err := syncer.Client.GetBeaconState(strconv.FormatUint(finalizedHeaderSlot, 10))
	if err != nil {
		return fmt.Errorf("download finalized beacon state at slot %d: %w", finalizedHeaderSlot, err)
	}

	err = store.WriteEntry(attestedHeaderSlot, finalizedHeaderSlot, attestedBeaconData, finalizedBeaconData)

	deletedSlots, err := store.PruneOldStates()
	log.WithField("deletedSlots", deletedSlots).Info("deleted old beacon states")

	return nil
}
