package cmd

import (
	"encoding/json"
	"fmt"
	"github.com/cbroglie/mustache"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
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
	InitialSync           syncer.InitialSync
	SyncCommitteeUpdate   syncer.SyncCommitteePeriodPayloadJSON
	FinalizedHeaderUpdate syncer.FinalizedHeaderPayloadJSON
	HeaderUpdate          syncer.HeaderUpdatePayloadJSON
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

	initialSync := syncer.InitialSync{}

	err = json.Unmarshal(initialSyncJSON, &initialSync)
	if err != nil {
		return err
	}

	syncCommitteeUpdateJSON, err := os.ReadFile(fmt.Sprintf("%s/%s_sync_committee_update.json", pathToBeaconTestFixtureFiles, spec))
	if err != nil {
		return err
	}

	syncCommitteeUpdate := syncer.SyncCommitteePeriodPayloadJSON{}

	err = json.Unmarshal(syncCommitteeUpdateJSON, &syncCommitteeUpdate)
	if err != nil {
		return err
	}

	finalizedHeaderJSON, err := os.ReadFile(fmt.Sprintf("%s/%s_finalized_header_update.json", pathToBeaconTestFixtureFiles, spec))
	if err != nil {
		return err
	}

	finalizedUpdate := syncer.FinalizedHeaderPayloadJSON{}

	err = json.Unmarshal(finalizedHeaderJSON, &finalizedUpdate)
	if err != nil {
		return err
	}

	headerUpdateJSON, err := os.ReadFile(fmt.Sprintf("%s/%s_block_update.json", pathToBeaconTestFixtureFiles, spec))
	if err != nil {
		return err
	}

	headerUpdate := syncer.HeaderUpdatePayloadJSON{}

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
