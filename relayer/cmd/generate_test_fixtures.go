package cmd

import (
	"encoding/json"
	"fmt"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"os"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"

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

	cmd.Flags().String("fallback-url", "", "fallback URL to use to download the beacon state")
	err = cmd.MarkFlagRequired("url")
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
	fallbackEndpoint, err := cmd.Flags().GetString("fallback-url")
	if err != nil {
		return fmt.Errorf("fallback url flag not set")
	}

	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	client := api.NewBeaconClient(endpoint, fallbackEndpoint, settings.SlotsInEpoch)
	syncer := syncer.New(client, settings)

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

	return nil
}

func writeFixtureJSONToFile(object interface{}, filename string) error {
	jsonObj, err := json.MarshalIndent(object, "", "  ")
	if err != nil {
		return fmt.Errorf("cannot marshall finalized checkpoint")
	}
	err = os.WriteFile(FixturesDir+filename, jsonObj, 0644)
	if err != nil {
		return fmt.Errorf("cannot write to file: %w", err)
	}

	return nil
}
