package cmd

import (
	"fmt"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/spf13/cobra"
)

func leafCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "leaf",
		Short: "dat leaf",
		Args:  cobra.ExactArgs(0),
		RunE:  LeafFn,
	}

	cmd.Flags().StringP("url", "u", "", "Polkadot URL")
	cmd.MarkFlagRequired("url")

	cmd.Flags().Uint64(
		"leaf-index",
		1,
		"Leaf index",
	)

	cmd.Flags().BytesHex(
		"block-hash",
		[]byte{},
		"Block Hash",
	)

	return cmd
}

func LeafFn(cmd *cobra.Command, _ []string) error {

	url, _ := cmd.Flags().GetString("url")
	leafIndex, _ := cmd.Flags().GetUint64("leaf-index")
	blockHashHex, _ := cmd.Flags().GetBytesHex("block-hash")

	var blockHash types.Hash
	copy(blockHash[:], blockHashHex[0:32])

	ctx := cmd.Context()

	conn := relaychain.NewConnection(url)
	err := conn.Connect(ctx)
	if err != nil {
		log.Error(err)
		return err
	}

	proof, err := conn.API().RPC.MMR.GenerateProof(leafIndex, blockHash)
	if err != nil {
		return err
	}

	simpleProof, err := merkle.ConvertToSimplifiedMMRProof(
		proof.BlockHash,
		uint64(proof.Proof.LeafIndex),
		proof.Leaf,
		uint64(proof.Proof.LeafCount),
		proof.Proof.Items,
	)

	leafEncoded, err := types.EncodeToBytes(simpleProof.Leaf)
	if err != nil {
		return err
	}
	leafHashBytes := (&keccak.Keccak256{}).Hash(leafEncoded)

	var leafHash types.H256
	copy(leafHash[:], leafHashBytes[0:32])

	root := merkle.CalculateMerkleRoot(&simpleProof, leafHash)
	if err != nil {
		return err
	}

	var actualMmrRoot types.H256

	mmrRootKey, err := types.CreateStorageKey(conn.Metadata(), "Mmr", "RootHash", nil, nil)
	if err != nil {
		return err
	}

	_, err = conn.API().RPC.State.GetStorage(mmrRootKey, &actualMmrRoot, blockHash)
	if err != nil {
		return err
	}

	actualParentHash, err := conn.API().RPC.Chain.GetBlockHash(uint64(proof.Leaf.ParentNumberAndHash.ParentNumber))
	if err != nil {
		return err
	}

	fmt.Printf("Leaf { ParentNumber: %v, ParentHash: %v, NextValidatorSetID: %v}\n",
		proof.Leaf.ParentNumberAndHash.ParentNumber,
		proof.Leaf.ParentNumberAndHash.Hash.Hex(),
		proof.Leaf.BeefyNextAuthoritySet.ID,
	)

	fmt.Printf("Actual ParentHash: %v %v\n", actualParentHash.Hex(), actualParentHash == proof.Leaf.ParentNumberAndHash.Hash)

	fmt.Printf("MMR Root: computed=%v actual=%v %v\n",
		root.Hex(), actualMmrRoot.Hex(), root == actualMmrRoot,
	)

	return nil
}
