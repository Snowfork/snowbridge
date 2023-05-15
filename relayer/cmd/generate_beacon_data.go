package cmd

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"os"

	"github.com/cbroglie/mustache"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	beaconjson "github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
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

	cmd.Flags().String("url", "http://127.0.0.1:9596", "Beacon URL")
	if err != nil {
		return nil
	}

	return cmd
}

func generateBeaconCheckPointCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-beacon-checkpoint",
		Short: "Generate beacon checkpoint.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateBeaconCheckPoint,
	}

	cmd.Flags().String("spec", "", "Valid values are mainnet or minimal")
	err := cmd.MarkFlagRequired("spec")
	if err != nil {
		return nil
	}

	cmd.Flags().String("url", "http://127.0.0.1:9596", "Beacon URL")
	if err != nil {
		return nil
	}

	return cmd
}

type Data struct {
	InitialSync           beaconjson.CheckPoint
	SyncCommitteeUpdate   beaconjson.SyncCommitteeUpdate
	FinalizedHeaderUpdate beaconjson.FinalizedHeaderUpdate
	HeaderUpdate          beaconjson.HeaderUpdate
}

const (
	pathToBeaconBenchmarkData    = "parachain/pallets/ethereum-beacon-client/src/benchmarking"
	pathToBenchmarkDataTemplate  = "parachain/templates/beacon_benchmarking_data.rs.mustache"
	pathToBeaconTestFixtureFiles = "parachain/pallets/ethereum-beacon-client/tests/fixtures"
)

func generateBeaconCheckPoint(cmd *cobra.Command, _ []string) error {
	err := func() error {
		spec, err := cmd.Flags().GetString("spec")
		if err != nil {
			return fmt.Errorf("get active spec: %w", err)
		}

		activeSpec, err := config.ToSpec(spec)
		if err != nil {
			return fmt.Errorf("get spec: %w", err)
		}

		endpoint, err := cmd.Flags().GetString("url")

		configFile := os.Getenv("output_dir") + "/beacon-relay.json"

		viper.SetConfigFile(configFile)
		if err := viper.ReadInConfig(); err != nil {
			return err
		}

		var conf config.Config
		err = viper.Unmarshal(&conf)
		if err != nil {
			return err
		}

		specSettings := conf.GetSpecSettingsBySpec(activeSpec)

		s := syncer.New(endpoint, specSettings.SlotsInEpoch, specSettings.EpochsPerSyncCommitteePeriod, specSettings.MaxSlotsPerHistoricalRoot, activeSpec)

		checkPointScale, err := s.GetCheckPoint()
		if err != nil {
			return fmt.Errorf("get initial sync: %w", err)
		}
		checkPointBytes, _ := types.EncodeToBytes(checkPointScale)
		// Call index for EthereumBeaconClient.check_point_update
		checkPointCallIndex := "0x3205"
		checkPointUpdateCall := checkPointCallIndex + hex.EncodeToString(checkPointBytes)
		fmt.Println(checkPointUpdateCall)
		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon checkpoint")
	}

	return nil
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

		endpoint, err := cmd.Flags().GetString("url")

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

		initialSyncScale, err := s.GetCheckPoint()
		if err != nil {
			return fmt.Errorf("get initial sync: %w", err)
		}
		initialSync := initialSyncScale.ToJSON()
		initialSyncHeaderSlot := initialSync.Header.Slot
		err = writeJSONToFile(initialSync, activeSpec.ToString()+"_initial_sync")
		if err != nil {
			return fmt.Errorf("write initial sync to file: %w", err)
		}
		log.Info("created initial sync file")

		log.Info("downloading beacon state, this can take a few minutes...")
		syncCommitteePeriod := s.ComputeSyncPeriodAtSlot(initialSyncHeaderSlot)
		syncCommitteeUpdateScale, err := s.GetSyncCommitteePeriodUpdate(syncCommitteePeriod)
		if err != nil {
			return fmt.Errorf("get sync committee update: %w", err)
		}
		syncCommitteeUpdate := syncCommitteeUpdateScale.Payload.ToJSON()
		err = writeJSONToFile(syncCommitteeUpdate, activeSpec.ToString()+"_sync_committee_update")
		if err != nil {
			return fmt.Errorf("write sync committee update to file: %w", err)
		}
		log.Info("created sync committee update file")

		log.Info("downloading beacon state, this can take a few minutes...")
		finalizedUpdateScale, err := s.GetFinalizedUpdate()
		if err != nil {
			return fmt.Errorf("get finalized header update: %w", err)
		}
		finalizedUpdate := finalizedUpdateScale.Payload.ToJSON()
		err = writeJSONToFile(finalizedUpdate, activeSpec.ToString()+"_finalized_header_update")
		if err != nil {
			return fmt.Errorf("write finalized header update to file: %w", err)
		}
		log.Info("created finalized header update file")

		blockUpdateSlot := uint64(finalizedUpdateScale.Payload.FinalizedHeader.Slot - 2)
		checkPoint := cache.Proof{
			FinalizedBlockRoot: finalizedUpdateScale.FinalizedHeaderBlockRoot,
			BlockRootsTree:     finalizedUpdateScale.BlockRootsTree,
			Slot:               uint64(finalizedUpdateScale.Payload.FinalizedHeader.Slot),
		}
		headerUpdateScale, err := s.GetNextHeaderUpdateBySlotWithAncestryProof(blockUpdateSlot, checkPoint)
		if err != nil {
			return fmt.Errorf("get header update: %w", err)
		}
		nextHeaderUpdateScale, err := s.GetNextHeaderUpdateBySlot(blockUpdateSlot + 1)
		if err != nil {
			return fmt.Errorf("get next header update to get sync aggregate: %w", err)
		}
		headerUpdateScale.Payload.SyncAggregate = nextHeaderUpdateScale.NextSyncAggregate
		headerUpdateScale.Payload.SignatureSlot = nextHeaderUpdateScale.Payload.BeaconHeader.Slot
		headerUpdate := headerUpdateScale.ToJSON()
		err = writeJSONToFile(headerUpdate, activeSpec.ToString()+"_header_update")
		if err != nil {
			return fmt.Errorf("write block update to file: %w", err)
		}

		log.Info("created header update file")

		log.Info("now updating benchmarking data files")

		// Rust file hexes require the 0x of hashes to be removed
		initialSync.RemoveLeadingZeroHashes()
		syncCommitteeUpdate.RemoveLeadingZeroHashes()
		finalizedUpdate.RemoveLeadingZeroHashes()
		headerUpdate.RemoveLeadingZeroHashes()

		data := Data{
			InitialSync:           initialSync,
			SyncCommitteeUpdate:   syncCommitteeUpdate,
			FinalizedHeaderUpdate: finalizedUpdate,
			HeaderUpdate:          headerUpdate,
		}

		log.WithFields(log.Fields{
			"location": pathToBeaconTestFixtureFiles,
			"spec":     activeSpec,
		}).Info("rendering file using mustache")

		rendered, err := mustache.RenderFile(pathToBenchmarkDataTemplate, data)
		filename := fmt.Sprintf("data_%s.rs", activeSpec)

		log.WithFields(log.Fields{
			"location": pathToBeaconBenchmarkData,
			"filename": filename,
		}).Info("writing result file")

		err = writeBenchmarkDataFile(filename, rendered)
		if err != nil {
			return err
		}

		log.WithField("spec", activeSpec).Info("done")

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon data")
	}

	return nil
}

func writeJSONToFile(data interface{}, filename string) error {
	file, _ := json.MarshalIndent(data, "", " ")

	f, err := os.OpenFile(fmt.Sprintf("%s/%s.json", pathToBeaconTestFixtureFiles, filename), os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)

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

func writeBenchmarkDataFile(filename, fileContents string) error {
	f, err := os.OpenFile(fmt.Sprintf("%s/%s", pathToBeaconBenchmarkData, filename), os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)

	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}

	defer f.Close()

	_, err = f.Write([]byte(fileContents))

	if err != nil {
		return fmt.Errorf("write to file: %w", err)
	}

	return nil
}
