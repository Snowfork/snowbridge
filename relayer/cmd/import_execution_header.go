package cmd

import (
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/spf13/cobra"
)

func importExecutionHeaderCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "import-execution-header",
		Short: "Import the provided execution header.",
		Args:  cobra.ExactArgs(0),
		RunE:  importExecutionHeaderFn,
	}

	//snowbridge-relay beacon import --beacon-header $$$$$$$$$$ --parachain-api wss://rococo-rpc.snowbridge.network --lodestar-api safdjdafjkdskfhj --private-key keyfile
	cmd.Flags().String("beacon-header", "", "Beacon header whose execution header that will be imported")
	cmd.MarkFlagRequired("beacon-header")

	cmd.Flags().String("parachain-endpoint", "", "Parachain API URL")
	cmd.MarkFlagRequired("parachain-endpoint")

	cmd.Flags().String("lodestar-endpoint", "", "Lodestar API URL")
	cmd.MarkFlagRequired("lodestar-endpoint")

	cmd.Flags().String("private-key-file", "", "File containing the private key for the relayer")
	cmd.MarkFlagRequired("private-key-file")

	return cmd
}

func importExecutionHeaderFn(cmd *cobra.Command, _ []string) error {

	err := func() error {
		_ = cmd.Context()

		beaconHeader, _ := cmd.Flags().GetString("beacon-header")

		log.WithField("hash", beaconHeader).Info("will be syncing execution header for beacon hash")

		lodestarEndpoint, _ := cmd.Flags().GetString("lodestar-endpoint")

		syncer := syncer.New(lodestarEndpoint, 32, 256)

		beaconHeaderHash := common.HexToHash(beaconHeader)

		block, err := syncer.Client.GetBeaconBlock(beaconHeaderHash)
		if err != nil {
			return fmt.Errorf("get block: %w", err)
		}

		log.WithField("slot", block.Data.Message.Slot).Info("found block at slot")

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error importing execution header")
	}

	return nil
}
