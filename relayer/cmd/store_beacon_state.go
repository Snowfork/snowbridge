package cmd

import (
	"database/sql"
	"fmt"
	"os"
	"strconv"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	_ "github.com/mattn/go-sqlite3"
)

const BeaconStateDir = "states"

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

	db, err := sql.Open("sqlite3", dbStoreLocation+"beacon-state")
	if err != nil {
		return err
	}
	defer db.Close()

	specSettings := conf.Source.Beacon.Spec
	beaconClient := api.NewBeaconClient(url, specSettings.SlotsInEpoch)
	syncer := syncer.New(beaconClient, specSettings)

	err = createDB(db)
	if err != nil {
		return err
	}

	err = createBeaconStateDir(dbStoreLocation + BeaconStateDir)
	if err != nil {
		return err
	}

	update, err := syncer.GetFinalizedUpdate()
	if err != nil {
		return err
	}

	attestedHeaderSlot := uint64(update.Payload.AttestedHeader.Slot)
	finalizedHeaderSlot := uint64(update.Payload.FinalizedHeader.Slot)
	attestedSyncPeriod := syncer.ComputeSyncPeriodAtSlot(attestedHeaderSlot)
	finalizedSyncPeriod := syncer.ComputeSyncPeriodAtSlot(finalizedHeaderSlot)
	attestedStateFileName := fmt.Sprintf("beacon_state_%d.ssz", attestedHeaderSlot)
	finalizedStateFileName := fmt.Sprintf("beacon_state_%d.ssz", finalizedHeaderSlot)

	attestedBeaconData, err := syncer.Client.GetBeaconState(strconv.FormatUint(attestedHeaderSlot, 10))
	if err != nil {
		return fmt.Errorf("download attested beacon state at slot %d: %w", attestedHeaderSlot, err)
	}
	finalizedBeaconData, err := syncer.Client.GetBeaconState(strconv.FormatUint(finalizedHeaderSlot, 10))
	if err != nil {
		return fmt.Errorf("download finalized beacon state at slot %d: %w", finalizedHeaderSlot, err)
	}

	err = writeToBeaconFile(dbStoreLocation+BeaconStateDir, attestedStateFileName, attestedBeaconData)
	if err != nil {
		return err
	}
	err = writeToBeaconFile(dbStoreLocation+BeaconStateDir, finalizedStateFileName, finalizedBeaconData)
	if err != nil {
		return err
	}

	err = storeUpdate(db, attestedHeaderSlot, finalizedHeaderSlot, attestedSyncPeriod, finalizedSyncPeriod, attestedStateFileName, finalizedStateFileName)
	if err != nil {
		return fmt.Errorf("store beacon update: %w", err)
	}

	return nil
}

func createBeaconStateDir(dirPath string) error {
	if _, err := os.Stat(dirPath); os.IsNotExist(err) {
		return os.MkdirAll(dirPath, os.ModePerm)
	}
	return nil
}

func createDB(db *sql.DB) error {
	sqlStmt := `CREATE TABLE IF NOT EXISTS beacon_state (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		attested_slot INTEGER NOT NULL,
		finalized_slot INTEGER NOT NULL,
		attested_sync_period INTEGER NOT NULL,
		finalized_sync_period INTEGER NOT NULL,
		attested_state_filename TEXT NOT NULL,
		finalized_state_filename TEXT NOT NULL,
		timestamp INTEGER DEFAULT (strftime('%s', 'now'))
	);`
	_, err := db.Exec(sqlStmt)
	if err != nil {
		return err
	}

	return nil
}

func storeUpdate(db *sql.DB, attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod uint64, attestedStateFileName, finalizedStateFileName string) error {
	insertStmt := `INSERT INTO beacon_state (attested_slot, finalized_slot,  attested_sync_period, finalized_sync_period, attested_state_filename, finalized_state_filename) VALUES (?, ?, ?, ?, ?, ?)`
	stmt, err := db.Prepare(insertStmt)
	if err != nil {
		return err
	}
	defer stmt.Close()

	_, err = stmt.Exec(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod, attestedStateFileName, finalizedStateFileName)
	if err != nil {
		return err
	}

	return nil
}

func writeToBeaconFile(dir, filename string, data []byte) error {
	err := os.WriteFile(dir+"/"+filename, data, 0644)
	if err != nil {
		return fmt.Errorf("cannot write to file: %w", err)
	}

	return nil
}
