package cmd

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/relays/beefy"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"github.com/spf13/cobra"
	"golang.org/x/sync/errgroup"
)

func scanBeefyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "scan-beefy",
		Short: "Scan beefy messages like the beefy relayer would.",
		Args:  cobra.ExactArgs(0),
		RunE:  ScanBeefyFn,
	}

	cmd.Flags().StringP("polkadot-url", "p", "ws://127.0.0.1:9944", "Polkadot URL.")
	cmd.Flags().Uint64P("beefy-block", "b", 0, "Beefy block.")
	cmd.MarkFlagRequired("beefy-block")
	return cmd
}

func ScanBeefyFn(cmd *cobra.Command, _ []string) error {
	ctx := cmd.Context()
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)

	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.DebugLevel)

	polkadotUrl, _ := cmd.Flags().GetString("polkadot-url")
	relaychainConn := relaychain.NewConnection(polkadotUrl)
	relaychainConn.Connect(ctx)

	config := beefy.SourceConfig{}
	polkadotListener := beefy.NewPolkadotListener(
		&config,
		relaychainConn,
	)

	beefyBlock, _ := cmd.Flags().GetUint64("beefy-block")
	logrus.WithFields(logrus.Fields{
		"polkadot-url": polkadotUrl,
		"beefy-block":  beefyBlock,
	}).Info("Connected to relaychain.")

	commitments, err := polkadotListener.Start(ctx, eg, beefyBlock)
	if err != nil {
		logrus.WithError(err).Fatalf("could not start")
	}

	eg.Go(func() error {
		for {
			select {
			case <-ctx.Done():
				return nil
			case commitment, ok := <-commitments:
				if !ok {
					return nil
				}
				logrus.WithField("commitment", commitment).Info("scanned commitment")
			}
		}
	})

	// Ensure clean termination upon SIGINT, SIGTERM
	eg.Go(func() error {
		notify := make(chan os.Signal, 1)
		signal.Notify(notify, syscall.SIGINT, syscall.SIGTERM)

		select {
		case <-ctx.Done():
			return ctx.Err()
		case sig := <-notify:
			logrus.WithField("signal", sig.String()).Info("Received signal")
			cancel()
		}

		return nil
	})

	err = eg.Wait()
	if err != nil {
		logrus.WithError(err).Fatal("Unhandled error")
		return err
	}

	return nil
}

func scanSingleBeefyBlockCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "scan-single-beefy-block",
		Short: "Scan a single block which contains beefy commitment",
		Args:  cobra.ExactArgs(0),
		RunE:  ScanSingleBeefyBlockFn,
	}

	cmd.Flags().StringP("polkadot-url", "p", "ws://127.0.0.1:9944", "Polkadot URL.")
	cmd.Flags().Uint64P("beefy-block", "b", 0, "Beefy block.")
	cmd.MarkFlagRequired("beefy-block")
	return cmd
}

func ScanSingleBeefyBlockFn(cmd *cobra.Command, _ []string) error {
	ctx := cmd.Context()
	log.SetOutput(logrus.WithFields(logrus.Fields{"logger": "stdlib"}).WriterLevel(logrus.InfoLevel))
	logrus.SetLevel(logrus.DebugLevel)

	polkadotUrl, _ := cmd.Flags().GetString("polkadot-url")
	relaychainConn := relaychain.NewConnection(polkadotUrl)
	err := relaychainConn.Connect(ctx)
	if err != nil {
		fmt.Errorf("connect: %w", err)
		return err
	}
	api := relaychainConn.API()
	// metadata := relaychainConn.Metadata()

	beefyBlockNumber, _ := cmd.Flags().GetUint64("beefy-block")
	logrus.WithFields(logrus.Fields{
		"polkadot-url": polkadotUrl,
		"beefy-block":  beefyBlockNumber,
	}).Info("Connected to relaychain.")

	beefyBlockHash, err := api.RPC.Chain.GetBlockHash(beefyBlockNumber)
	if err != nil {
		return fmt.Errorf("fetch hash: %w", err)
	}

	beefyBlock, err := api.RPC.Chain.GetBlock(beefyBlockHash)
	if err != nil {
		return fmt.Errorf("fetch block: %w", err)
	}

	var commitment *types.SignedCommitment
	for j := range beefyBlock.Justifications {
		sc := types.OptionalSignedCommitment{}
		if beefyBlock.Justifications[j].EngineID() == "BEEF" {
			err := types.DecodeFromBytes(beefyBlock.Justifications[j].Payload(), &sc)
			if err != nil {
				return fmt.Errorf("decode BEEFY signed commitment: %w", err)
			}
			ok, value := sc.Unwrap()
			if ok {
				commitment = &value
			}
		}
	}
	if commitment == nil {
		return fmt.Errorf("beefy block without a valid commitment")
	}
	if len(commitment.Signatures) == 0 {
		return fmt.Errorf("no signature in the commitment")
	}
	var emptyNum uint
	var errNum uint
	var revertedNum uint
	for _, s := range commitment.Signatures {
		ok, beefySig := s.Unwrap()
		if !ok {
			emptyNum++
			continue
		}
		sBefore := util.BytesToHexString(beefySig[32:64])
		_, _, s, reverted, err := beefy.CleanSignature(beefySig)
		if err != nil {
			logrus.WithError(err).Warn("cleanSignature")
			errNum++
		}
		sAfter := util.BytesToHexString(s[:])
		if reverted {
			revertedNum++
			logrus.Info(fmt.Sprintf("s is reverted, before clean:%s, after clean:%s", sBefore, sAfter))
		}
	}
	logrus.Info(fmt.Sprintf("number of total signatures:%d,empty signatures:%d,invalid signatures:%d,reverted signatures:%d", len(commitment.Signatures), emptyNum, errNum, revertedNum))
	return nil
}
