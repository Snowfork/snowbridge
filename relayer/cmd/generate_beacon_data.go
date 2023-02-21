package cmd

import (
	"encoding/json"
	"fmt"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"os"
)

func generateBeaconDataCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-beacon-data",
		Short: "Generate beacon data.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateBeaconData,
	}

	cmd.Flags().String("spec", "", "Valid values are mainnet or minimal")
	err := cmd.MarkFlagRequired("spec")
	if err != nil {
		return nil
	}

	return cmd
}

func generateBeaconData(cmd *cobra.Command, _ []string) error {
	err := func() error {
		spec, err := cmd.Flags().GetString("spec")
		if err != nil {
			return fmt.Errorf("get active spec: %w", err)
		}

		activeSpec, err := config.ToSpec(spec)
		if err != nil {
			return fmt.Errorf("get spec: %w", err)
		}

		endpoint := ""

		//if activeSpec.IsMinimal() {
		endpoint = "http://127.0.0.1:9596"
		//} else {
		//	endpoint = "https://lodestar-goerli.chainsafe.io"
		//}

		viper.SetConfigFile("core/packages/test/config/beacon-relay.json")
		if err := viper.ReadInConfig(); err != nil {
			return err
		}

		var conf config.Config
		err = viper.Unmarshal(&conf)
		if err != nil {
			return err
		}

		specSettings := conf.GetSpecSettingsBySpec(activeSpec)

		log.WithFields(log.Fields{"spec": activeSpec, "endpoint": endpoint}).Info("connecting to beacon API")

		s := syncer.New(endpoint, specSettings.SlotsInEpoch, specSettings.EpochsPerSyncCommitteePeriod, specSettings.MaxSlotsPerHistoricalRoot, activeSpec)

		initialSync, err := s.GetInitialSync()
		if err != nil {
			return fmt.Errorf("get initial sync: %w", err)
		}

		initialSyncHeaderSlot := initialSync.Header.Slot

		syncCommitteePeriod := s.ComputeSyncPeriodAtSlot(initialSyncHeaderSlot)

		err = writeJSONToFile(initialSync, activeSpec.ToString()+"_initial_sync")
		if err != nil {
			return fmt.Errorf("write initial sync to file: %w", err)
		}

		log.Info("created initial sync file")

		committeeUpdate, err := s.GetSyncCommitteePeriodUpdate(syncCommitteePeriod)
		if err != nil {
			return fmt.Errorf("get sync committee update: %w", err)
		}

		err = writeJSONToFile(committeeUpdate.Payload.ToJSON(), activeSpec.ToString()+"_sync_committee_update")
		if err != nil {
			return fmt.Errorf("write sync committee update to file: %w", err)
		}

		log.Info("created sync committee update file")

		finalizedHeaderUpdate, err := s.GetFinalizedUpdate()
		if err != nil {
			return fmt.Errorf("get finalized header update: %w", err)
		}

		err = writeJSONToFile(finalizedHeaderUpdate.Payload.ToJSON(), activeSpec.ToString()+"_finalized_header_update")
		if err != nil {
			return fmt.Errorf("write finalized header update to file: %w", err)
		}

		log.Info("created finalized header update file")

		blockUpdateSlot := uint64(finalizedHeaderUpdate.Payload.FinalizedHeader.Slot - 2)

		checkPoint := cache.Proof{
			FinalizedBlockRoot: finalizedHeaderUpdate.FinalizedHeaderBlockRoot,
			BlockRootsTree:     finalizedHeaderUpdate.BlockRootsTree,
			Slot:               uint64(finalizedHeaderUpdate.Payload.FinalizedHeader.Slot),
		}

		blockUpdate, err := s.GetNextHeaderUpdateBySlotWithAncestryProof(blockUpdateSlot, checkPoint)
		if err != nil {
			return fmt.Errorf("get header update: %w", err)
		}
		nextBlockUpdate, err := s.GetNextHeaderUpdateBySlot(blockUpdateSlot + 1)
		if err != nil {
			return fmt.Errorf("get next header update to get sync aggregate: %w", err)
		}

		log.WithField("nextBlockUpdate", nextBlockUpdate.Block.Body.SyncAggregate).Info("next block")

		blockUpdate.SyncAggregate = nextBlockUpdate.Block.Body.SyncAggregate
		blockUpdate.SignatureSlot = nextBlockUpdate.Block.Slot

		err = writeJSONToFile(blockUpdate.ToJSON(), activeSpec.ToString()+"_block_update")
		if err != nil {
			return fmt.Errorf("write block update to file: %w", err)
		}

		log.Info("created block update")

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon data")
	}

	return nil
}

func writeJSONToFile(data interface{}, filename string) error {
	file, _ := json.MarshalIndent(data, "", " ")

	f, err := os.OpenFile("parachain/pallets/ethereum-beacon-client/tests/fixtures/"+filename+".json", os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)

	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}

	defer f.Close()

	_, err = f.Write(file)

	if err != nil {
		return fmt.Errorf("write to file: %w", err)
	}

	return nil
}
