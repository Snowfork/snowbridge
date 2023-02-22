package cmd

import (
	"encoding/json"
	"fmt"
	"github.com/cbroglie/mustache"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	beaconjson "github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/spf13/cobra"
	"os"
)

func generateBeaconBenchmarkDataCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-beacon-benchmark-data",
		Short: "Generate beacon benchmark data.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateBeaconBenchmarkData,
	}

	return cmd
}

type Data struct {
	InitialSync           beaconjson.InitialSync
	SyncCommitteeUpdate   beaconjson.SyncCommitteeUpdate
	FinalizedHeaderUpdate beaconjson.FinalizedHeaderUpdate
	HeaderUpdate          beaconjson.HeaderUpdate
}

const (
	pathToBeaconBenchmarkData   = "parachain/pallets/ethereum-beacon-client/src/benchmarking"
	pathToBenchmarkDataTemplate = "parachain/templates/beacon_benchmarking_data.rs.mustache"
)

func generateBeaconBenchmarkData(cmd *cobra.Command, _ []string) error {
	err := func() error {
		err := generateBenchmarkDataFiles(config.Minimal)
		if err != nil {
			return fmt.Errorf("generate minimal benchmark data files: %w", err)
		}

		err = generateBenchmarkDataFiles(config.Mainnet)
		if err != nil {
			return fmt.Errorf("generate mainnet benchmark data files: %w", err)
		}

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error generating beacon data")
	}

	return nil
}

func generateBenchmarkDataFiles(spec config.ActiveSpec) error {
	log.WithFields(log.Fields{
		"location": pathToBeaconTestFixtureFiles,
		"spec":     spec,
	}).Info("reading beacon test fixture data")

	initialSyncJSON, err := os.ReadFile(fmt.Sprintf("%s/%s_initial_sync.json", pathToBeaconTestFixtureFiles, spec))
	if err != nil {
		return err
	}

	initialSync := beaconjson.InitialSync{}

	err = json.Unmarshal(initialSyncJSON, &initialSync)
	if err != nil {
		return err
	}

	syncCommitteeUpdateJSON, err := os.ReadFile(fmt.Sprintf("%s/%s_sync_committee_update.json", pathToBeaconTestFixtureFiles, spec))
	if err != nil {
		return err
	}

	syncCommitteeUpdate := beaconjson.SyncCommitteeUpdate{}

	err = json.Unmarshal(syncCommitteeUpdateJSON, &syncCommitteeUpdate)
	if err != nil {
		return err
	}

	finalizedHeaderJSON, err := os.ReadFile(fmt.Sprintf("%s/%s_finalized_header_update.json", pathToBeaconTestFixtureFiles, spec))
	if err != nil {
		return err
	}

	finalizedUpdate := beaconjson.FinalizedHeaderUpdate{}

	err = json.Unmarshal(finalizedHeaderJSON, &finalizedUpdate)
	if err != nil {
		return err
	}

	headerUpdateJSON, err := os.ReadFile(fmt.Sprintf("%s/%s_block_update.json", pathToBeaconTestFixtureFiles, spec))
	if err != nil {
		return err
	}

	headerUpdate := beaconjson.HeaderUpdate{}

	err = json.Unmarshal(headerUpdateJSON, &headerUpdate)
	if err != nil {
		return err
	}

	log.Info("removing leading 0x from hex string values")

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
		"spec":     spec,
	}).Info("rendering file using mustache")

	rendered, err := mustache.RenderFile(pathToBenchmarkDataTemplate, data)

	filename := fmt.Sprintf("data_%s.rs", spec)

	log.WithFields(log.Fields{
		"location": pathToBeaconBenchmarkData,
		"filename": filename,
	}).Info("writing result file")

	f, err := os.OpenFile(fmt.Sprintf("%s/%s", pathToBeaconBenchmarkData, filename), os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)

	if err != nil {
		return fmt.Errorf("create file: %w", err)
	}

	defer f.Close()

	_, err = f.Write([]byte(rendered))

	if err != nil {
		return fmt.Errorf("write to file: %w", err)
	}

	log.WithField("spec", spec).Info("done")

	return nil
}
