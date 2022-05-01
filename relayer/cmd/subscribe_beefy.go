package cmd

import (
	"context"
	"fmt"

	"github.com/sirupsen/logrus"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/spf13/cobra"
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

	return cmd
}

func SubBeefyFn(cmd *cobra.Command, _ []string) error {
	subBeefyJustifications(cmd.Context(), cmd)
	return nil
}

func subBeefyJustifications(ctx context.Context, cmd *cobra.Command) error {
	url, _ := cmd.Flags().GetString("url")

	conn := relaychain.NewConnection(url)
	err := conn.Connect(ctx)
	if err != nil {
		log.Error(err)
		return err
	}

	sub, err := conn.API().RPC.Beefy.SubscribeJustifications()
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case commitment := <-sub.Chan():

			blockNumber := commitment.Commitment.BlockNumber

			if len(commitment.Signatures) == 0 {
				log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}

			blockHash, err := conn.API().RPC.Chain.GetBlockHash(uint64(blockNumber))
			if err != nil {
				return err
			}

			proof, err := conn.API().RPC.MMR.GenerateProof(uint64(blockNumber-1), blockHash)
			if err != nil {
				return err
			}

			fmt.Printf("Commitment { BlockNumber: %v ValidatorSetID: %v}; Leaf { ParentNumber: %v, NextValidatorSetID: %v }\n",
				blockNumber, commitment.Commitment.ValidatorSetID, proof.Leaf.ParentNumberAndHash.ParentNumber, proof.Leaf.BeefyNextAuthoritySet.ID,
			)

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

func GetMMRLeafForBlock(blockNumber uint64, blockHash types.Hash, conn *relaychain.Connection) {
	log.WithFields(logrus.Fields{
		"blockNumber": blockNumber,
		"blockHash":   blockHash.Hex(),
	}).Info("Getting MMR Leaf for block...")
	proofResponse, err := conn.API().RPC.MMR.GenerateProof(blockNumber, blockHash)
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
