package cmd

import (
	"fmt"
	"io/ioutil"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/spf13/cobra"
	"golang.org/x/sync/errgroup"
)

func importExecutionHeaderCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "import-execution-header",
		Short: "Import the provided execution header.",
		Args:  cobra.ExactArgs(0),
		RunE:  importExecutionHeaderFn,
	}

	cmd.Flags().String("beacon-header", "", "Beacon header hash whose execution header will be imported")
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
		ctx := cmd.Context()

		eg, ctx := errgroup.WithContext(ctx)

		parachainEndpoint, _ := cmd.Flags().GetString("parachain-endpoint")
		privateKeyFile, _ := cmd.Flags().GetString("private-key-file")
		lodestarEndpoint, _ := cmd.Flags().GetString("lodestar-endpoint")
		beaconHeader, _ := cmd.Flags().GetString("beacon-header")

		keypair, err := getKeyPair(privateKeyFile)
		if err != nil {
			return fmt.Errorf("get keypair from file: %w", err)
		}

		paraconn := parachain.NewConnection(parachainEndpoint, keypair.AsKeyringPair())
		err = paraconn.Connect(ctx)
		if err != nil {
			return fmt.Errorf("connect to parachain: %w", err)
		}

		writer := parachain.NewParachainWriter(paraconn, 32)
		err = writer.Start(ctx, eg)
		if err != nil {
			return fmt.Errorf("start parachain conn: %w", err)
		}

		log.WithField("hash", beaconHeader).Info("will be syncing execution header for beacon hash")

		syncer := syncer.New(lodestarEndpoint, 32, 256)

		beaconHeaderHash := common.HexToHash(beaconHeader)

		update, err := syncer.GetHeaderUpdate(beaconHeaderHash)
		if err != nil {
			return fmt.Errorf("get header update: %w", err)
		}
		log.WithField("slot", update.Block.Slot).Info("found block at slot")

		syncAggregate, signatureSlot, err := syncer.GetSyncAggregateForSlot(uint64(update.Block.Slot) + 1)
		if err != nil {
			return fmt.Errorf("get sync aggregate: %w", err)
		}
		log.Info("found sync aggregate")

		update.SyncAggregate = syncAggregate
		update.SignatureSlot = signatureSlot

		err = writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.import_execution_header", update)
		if err != nil {
			return fmt.Errorf("write to parachain: %w", err)
		}
		log.Info("imported execution header")

		return nil
	}()
	if err != nil {
		log.WithError(err).Error("error importing execution header")
	}

	return nil
}

func getKeyPair(privateKeyFile string) (*sr25519.Keypair, error) {
	var cleanedKeyURI string
	content, err := ioutil.ReadFile(privateKeyFile)
	if err != nil {
		return nil, fmt.Errorf("cannot read key file: %w", err)
	}
	cleanedKeyURI = strings.TrimSpace(string(content))
	keypair, err := sr25519.NewKeypairFromSeed(cleanedKeyURI, 42)
	if err != nil {
		return nil, fmt.Errorf("parse private key URI: %w", err)
	}

	return keypair, nil
}
