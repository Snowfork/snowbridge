package testutil

import (
	"encoding/json"
	"log"
	"os"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
)

const FixturesDir = "fixtures/"

func main() {
	settings := config.SpecSettings{
		SlotsInEpoch:                 32,
		EpochsPerSyncCommitteePeriod: 256,
		DenebForkEpoch:               0,
	}
	client := api.NewBeaconClient("http://localhost:3500", settings.SlotsInEpoch)

	//syncer := syncer.New(client, settings)

	finalizedCheckpoint, err := client.GetFinalizedCheckpoint()
	if err != nil {
		log.Fatal("cannot get finalized checkpoint")
	}
	finalizedCheckpointJSON, err := json.Marshal(finalizedCheckpoint)
	if err != nil {
		log.Fatal("cannot marshall finalized checkpoint")
	}
	err = os.WriteFile(FixturesDir+"finalized_checkpoint.json", finalizedCheckpointJSON, 0644)
	if err != nil {
		log.Fatal("cannot write to file")
	}
}
