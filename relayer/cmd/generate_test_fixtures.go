package cmd

import (
	"encoding/json"
	"fmt"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"
	"os"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/util"

	"github.com/spf13/cobra"
)

const FixturesDir = "relays/testutil/fixtures/"

func generateTestFixtures() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-test-fixtures",
		Short: "Import the provided execution header.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateFixtures,
	}

	cmd.Flags().String("url", "", "URL to generate test fixtures from")
	err := cmd.MarkFlagRequired("url")
	if err != nil {
		return nil
	}

	return cmd
}

func generateFixtures(cmd *cobra.Command, _ []string) error {
	endpoint, err := cmd.Flags().GetString("url")
	if err != nil {
		return fmt.Errorf("url flag not set")
	}

	store := store.New("./", 100)
	store.Connect()
	defer store.Close()

	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	client := api.NewBeaconClient(endpoint, settings.SlotsInEpoch)
	syncer := syncer.New(client, settings, &store)

	finalizedCheckpoint, err := client.GetLatestFinalizedUpdate()
	if err != nil {
		return err
	}
	err = writeFixtureJSONToFile(finalizedCheckpoint, "finalized_update.json")
	if err != nil {
		return err
	}

	currentSlot, err := util.ToUint64(finalizedCheckpoint.Data.SignatureSlot)
	if err != nil {
		return err
	}

	currentPeriod := syncer.ComputeSyncPeriodAtSlot(currentSlot)

	syncCommitteeUpdate, err := client.GetSyncCommitteePeriodUpdate(currentPeriod - 2)
	if err != nil {
		return err
	}
	err = writeFixtureJSONToFile(syncCommitteeUpdate, "sync_committee_update.json")
	if err != nil {
		return err
	}

	attestedHeaderSlot, err := util.ToUint64(syncCommitteeUpdate.Data.AttestedHeader.Beacon.Slot)
	if err != nil {
		return err
	}
	checkpointSlot := syncer.CalculateNextCheckpointSlot(attestedHeaderSlot)

	headerAtSlot, err := client.GetHeaderBySlot(checkpointSlot)
	if err != nil {
		return err
	}

	err = writeFixtureJSONToFile(headerAtSlot, fmt.Sprintf("header_at_slot_%d.json", checkpointSlot))
	if err != nil {
		return err
	}

	beaconState, err := client.GetBeaconState(syncCommitteeUpdate.Data.FinalizedHeader.Beacon.Slot)
	if err != nil {
		return err
	}

	err = writeToFile(fmt.Sprintf("beacon_state_%s.ssz", syncCommitteeUpdate.Data.FinalizedHeader.Beacon.Slot), beaconState)
	if err != nil {
		return err
	}

	return nil
}

func writeFixtureJSONToFile(object interface{}, filename string) error {
	jsonObj, err := json.MarshalIndent(object, "", "  ")
	if err != nil {
		return fmt.Errorf("cannot marshall finalized checkpoint")
	}

	return writeToFile(filename, jsonObj)
}

func writeToFile(filename string, data []byte) error {
	err := os.WriteFile(FixturesDir+filename, data, 0644)
	if err != nil {
		return fmt.Errorf("cannot write to file: %w", err)
	}

	return nil
}
