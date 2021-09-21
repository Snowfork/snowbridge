package parachain

import (
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type ParaBlockWithDigest struct {
	BlockNumber         uint64
	DigestItemsWithData []DigestItemWithData
}

type ParaBlockWithProofs struct {
	Block            ParaBlockWithDigest
	MMRProof merkle.SimplifiedMMRProof
	MMRRootHash      types.Hash
	Header           types.Header
	MerkleProofData  MerkleProofData
}

type DigestItemWithData struct {
	DigestItem parachain.AuxiliaryDigestItem
	Data       types.StorageDataRaw
}

type MessagePackage struct {
	channelID         parachain.ChannelID
	commitmentHash    types.H256
	commitmentData    types.StorageDataRaw
	paraHead          types.Header
	merkleProofData   MerkleProofData
	paraId            uint32
	mmrRootHash       types.Hash
	simplifiedMMRProof merkle.SimplifiedMMRProof
}

func CreateMessagePackages(paraBlocks []ParaBlockWithProofs, mmrLeafCount uint64, paraID uint32) ([]MessagePackage, error) {
	var messagePackages []MessagePackage

	for _, block := range paraBlocks {
		for _, item := range block.Block.DigestItemsWithData {
			commitmentHash := item.DigestItem.AsCommitment.Hash
			commitmentData := item.Data
			messagePackage := MessagePackage{
				item.DigestItem.AsCommitment.ChannelID,
				commitmentHash,
				commitmentData,
				block.Header,
				block.MerkleProofData,
				paraID,
				block.MMRRootHash,
				block.MMRProof,
			}
			messagePackages = append(messagePackages, messagePackage)
		}
	}

	return messagePackages, nil
}
