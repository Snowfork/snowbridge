package cmd

import (
	"context"

	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/store"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func subBeefyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sub-beefy",
		Short: "Subscribe beefy messages",
		Args:  cobra.ExactArgs(0),
		RunE:  SubBeefyFn,
	}

	cmd.Flags().StringP("url", "u", "", "Polkadot URL")
	cmd.MarkFlagRequired("url")

	cmd.Flags().UintP(
		"para-id",
		"i",
		1000,
		"Parachain ID",
	)
	cmd.MarkFlagRequired("para-id")

	viper.BindPFlags(cmd.Flags())
	return cmd
}

func SubBeefyFn(cmd *cobra.Command, _ []string) error {
	paraID := viper.GetUint32("para-id")
	subBeefyJustifications(cmd.Context(), paraID)
	return nil
}

func subBeefyJustifications(ctx context.Context, paraID uint32) error {
	relaychainConn := relaychain.NewConnection(viper.GetString("url"))
	err := relaychainConn.Connect(ctx)
	if err != nil {
		log.Error(err)
		return err
	}

	ch := make(chan interface{})

	log.Info("Subscribing to beefy justifications")
	sub, err := relaychainConn.API().Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
	if err != nil {
		panic(err)
	}
	defer sub.Unsubscribe()

	for {
		select {
		case msg := <-ch:

			signedCommitment := &store.SignedCommitment{}
			err := types.DecodeFromHexString(msg.(string), signedCommitment)
			if err != nil {
				log.WithError(err).Error("Failed to decode BEEFY commitment messages")
			}

			blockNumber := signedCommitment.Commitment.BlockNumber

			log.WithField("commitmentBlockNumber", blockNumber).Info("Witnessed a new BEEFY commitment: \n")
			if len(signedCommitment.Signatures) == 0 {
				log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}
			log.WithField("blockNumber", blockNumber+1).Info("Getting hash for next block")
			nextBlockHash, err := relaychainConn.API().RPC.Chain.GetBlockHash(uint64(blockNumber + 1))
			if err != nil {
				log.WithError(err).Error("Failed to get block hash")
			}
			log.WithField("blockHash", nextBlockHash.Hex()).Info("Got blockhash")
			GetMMRLeafForBlock(uint64(blockNumber), nextBlockHash, relaychainConn)
			heads, err := fetchParaHeads(relaychainConn, nextBlockHash)

			var ourParahead types.Header
			if err := types.DecodeFromBytes(heads[paraID], &ourParahead); err != nil {
				log.WithError(err).Error("Failed to decode Header")
				return err
			}

			log.WithFields(logrus.Fields{
				"allParaheads": heads,
				"ourParahead":  ourParahead,
			}).Info("Got all para heads")

			// TODO6 - Update all above code to make sure to check all new parachain blocks that have been added to the MMR
			// when there is a new beefy justification, not just the newest parachain block in the MMR
		}
	}
}

// Copied over from relaychain.Connection
func fetchParaHeads(co *relaychain.Connection, blockHash types.Hash) (map[uint32][]byte, error) {

	keyPrefix := types.CreateStorageKeyPrefix("Paras", "Heads")

	keys, err := co.API().RPC.State.GetKeys(keyPrefix, blockHash)
	if err != nil {
		log.WithError(err).Error("Failed to get all parachain keys")
		return nil, err
	}

	changeSets, err := co.API().RPC.State.QueryStorageAt(keys, blockHash)
	if err != nil {
		log.WithError(err).Error("Failed to get all parachain headers")
		return nil, err
	}

	heads := make(map[uint32][]byte)

	for _, changeSet := range changeSets {
		for _, change := range changeSet.Changes {
			if change.StorageData.IsNone() {
				continue
			}

			var paraID uint32

			if err := types.DecodeFromBytes(change.StorageKey[40:], &paraID); err != nil {
				log.WithError(err).Error("Failed to decode parachain ID")
				return nil, err
			}

			_, headDataWrapped := change.StorageData.Unwrap()

			var headData types.Bytes
			if err := types.DecodeFromBytes(headDataWrapped, &headData); err != nil {
				log.WithError(err).Error("Failed to decode HeadData wrapper")
				return nil, err
			}

			heads[paraID] = headData
		}
	}

	return heads, nil
}

func GetMMRLeafForBlock(blockNumber uint64, blockHash types.Hash, relaychainConn *relaychain.Connection) {
	log.WithFields(logrus.Fields{
		"blockNumber": blockNumber,
		"blockHash":   blockHash.Hex(),
	}).Info("Getting MMR Leaf for block...")
	proofResponse, err := relaychainConn.API().RPC.MMR.GenerateProof(blockNumber, blockHash)
	if err != nil {
		log.WithError(err).Error("Failed to generate mmr proof")
	}

	log.WithFields(logrus.Fields{
		"BlockHash":                       proofResponse.BlockHash.Hex(),
		"Leaf.ParentNumber":               proofResponse.Leaf.ParentNumberAndHash.ParentNumber,
		"Leaf.Hash":                       proofResponse.Leaf.ParentNumberAndHash.Hash.Hex(),
		"Leaf.ParachainHeads":             proofResponse.Leaf.ParachainHeads.Hex(),
		"Leaf.BeefyNextAuthoritySet.ID":   proofResponse.Leaf.BeefyNextAuthoritySet.ID,
		"Leaf.BeefyNextAuthoritySet.Len":  proofResponse.Leaf.BeefyNextAuthoritySet.Len,
		"Leaf.BeefyNextAuthoritySet.Root": proofResponse.Leaf.BeefyNextAuthoritySet.Root.Hex(),
		"Proof.LeafIndex":                 proofResponse.Proof.LeafIndex,
		"Proof.LeafCount":                 proofResponse.Proof.LeafCount,
	}).Info("Generated MMR Proof")
}
