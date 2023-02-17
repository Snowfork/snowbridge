package cmd

import (
	"encoding/json"
	"fmt"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/spf13/cobra"
	"os"
)

func generateBeaconDataCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "generate-beacon-data",
		Short: "Generate beacon data.",
		Args:  cobra.ExactArgs(0),
		RunE:  generateBeaconData,
	}

	return cmd
}

func generateBeaconData(cmd *cobra.Command, _ []string) error {
	err := func() error {
		s := syncer.New("http://127.0.0.1:9596", 8, 8, 64, config.Minimal)

		initialSync, err := s.GetInitialSync()
		if err != nil {
			return fmt.Errorf("get initial sync: %w", err)
		}

		file, _ := json.MarshalIndent(initialSync, "", " ")

		f, err := os.Create("data.txt")

		if err != nil {
			log.Fatal(err)
		}

		defer f.Close()

		_, err2 := f.WriteString(string(file))

		if err2 != nil {
			log.Fatal(err2)
		}

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error importing execution header")
	}

	return nil
}
