package testutil

import (
	"encoding/json"
	"fmt"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"os"
)

const Dirname = "fixtures/"

func GetSyncCommitteeUpdate() (api.SyncCommitteePeriodUpdateResponse, error) {
	var update api.SyncCommitteePeriodUpdateResponse

	data, err := loadFile("older_sync_committee_update.json")
	if err != nil {
		return update, fmt.Errorf("error reading file: %w", err)
	}

	err = json.Unmarshal(data, &update)
	if err != nil {
		return update, fmt.Errorf("error unmarshalling json: %w", err)
	}

	return update, nil
}

func GetFinalizedUpdate() (api.LatestFinalisedUpdateResponse, error) {
	var update api.LatestFinalisedUpdateResponse

	data, err := loadFile("finalized_update.json")
	if err != nil {
		return update, fmt.Errorf("error reading file: %w", err)
	}

	err = json.Unmarshal(data, &update)
	if err != nil {
		return update, fmt.Errorf("error unmarshalling json: %w", err)
	}

	return update, nil
}

func loadFile(filename string) ([]byte, error) {
	jsonData, err := os.ReadFile(Dirname + filename)
	if err != nil {
		return nil, fmt.Errorf("error reading file")
	}

	return jsonData, nil

}
