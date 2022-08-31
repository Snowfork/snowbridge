package cmd

import (
	"errors"
	"fmt"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/relays/parachain"
	"github.com/spf13/cobra"
)

func beefyProofCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "beefy-proof",
		Short: "Prove a block using beefy.",
		Args:  cobra.ExactArgs(0),
		RunE:  BeefyProofFn,
	}

	cmd.Flags().StringP("url", "u", "", "Polkadot URL")
	cmd.MarkFlagRequired("url")

	cmd.Flags().BytesHex(
		"beefy-block-hash",
		[]byte{},
		"Latest beefy block",
	)
	cmd.MarkFlagRequired("beefy-block-hash")

	cmd.Flags().Uint64(
		"relaychain-block",
		0,
		"The relaychain block you are trying to prove. i.e. Leaf Index.",
	)
	cmd.MarkFlagRequired("relaychain-block")

	cmd.Flags().Uint32(
		"parachain-id",
		0,
		"The parachain id for the block you are trying to prove.",
	)
	cmd.MarkFlagRequired("parachain-id")

	cmd.Flags().Uint64(
		"parachain-block",
		0,
		"The parachain block you are trying to prove. i.e. The block containing the message bundle.",
	)
	cmd.MarkFlagRequired("parachain-block")
	return cmd
}

func BeefyProofFn(cmd *cobra.Command, _ []string) error {
	ctx := cmd.Context()

	url, _ := cmd.Flags().GetString("url")
	conn := relaychain.NewConnection(url)
	err := conn.Connect(ctx)
	if err != nil {
		log.WithError(err).Error("Cannot connect.")
		return err
	}

	beefyBlockHashHex, _ := cmd.Flags().GetBytesHex("beefy-block-hash")
	if len(beefyBlockHashHex) != 32 {
		log.Error("Incorrect lenght of beefy block hash.")
		return errors.New("incorrect block hash length")
	}
	var beefyBlockHash types.Hash
	copy(beefyBlockHash[:], beefyBlockHashHex[0:32])

	relayChainBlock, _ := cmd.Flags().GetUint64("relaychain-block")
	mmrProof, err := conn.GenerateProofForBlock(relayChainBlock, beefyBlockHash, 0)
	if err != nil {
		log.WithError(err).Error("Cannot connect.")
		return err
	}

	paraId, _ := cmd.Flags().GetUint32("parachain-id")
	parachainBlock, _ := cmd.Flags().GetUint64("parachain-block")

	relayChainBlockHash, err := conn.API().RPC.Chain.GetBlockHash(relayChainBlock)
	if err != nil {
		log.WithError(err).Error("Cannot fetch parachain block hash.")
		return err
	}

	paraHeads, err := conn.FetchParaHeads(relayChainBlockHash)
	if err != nil {
		log.WithError(err).Error("Cannot fetch para heads.")
		return err
	}

	log.WithField("relayChainBlockHash", relayChainBlockHash).WithField("paraId", paraId).Info("ParaHeads")
	if _, ok := paraHeads[paraId]; !ok {
		return fmt.Errorf("snowbridge is not a registered parachain")
	}

	paraHeadsAsSlice := make([]relaychain.ParaHead, 0, len(paraHeads))
	for _, v := range paraHeads {
		paraHeadsAsSlice = append(paraHeadsAsSlice, v)
	}

	// mmrRootHash, err := conn.GetMMRRootHash(beefyBlockHash)
	// if err != nil {
	// 	log.WithError(err).Error("Cannot get MMR root hash.")
	// 	return err
	// }
	// _ = mmrRootHash

	merkleProofData, err := parachain.CreateParachainMerkleProof(paraHeadsAsSlice, paraId)
	if err != nil {
		log.WithError(err).Error("Cannot create merkle proof.")
		return err
	}

	log.WithFields(log.Fields{
		"parachainId":           paraId,
		"relaychainBlockHash":   relayChainBlockHash.Hex(),
		"relaychainBlockNumber": relayChainBlock,
		"parachainBlockNumber":  parachainBlock,
		"paraHeads":             paraHeadsAsSlice,
	}).Info("Generated proof input for parachain block.")

	log.WithFields(log.Fields{
		"mmrProofParachainHeads":           mmrProof.Leaf.ParachainHeads.Hex(),
		"mmrProofParentNumberAndHash":      mmrProof.Leaf.ParentNumberAndHash,
		"computedProofParachainHeads":      merkleProofData.Root.Hex(),
		"computedProofParentNumberAndHash": types.ParentNumberAndHash{ParentNumber: types.U32(relayChainBlock), Hash: relayChainBlockHash},
	}).Info("Complete.")

	return nil
}
